use rand::{SeedableRng, rngs::StdRng};

use crate::{
    ClipperCommand, ClipperState, Ellipsoid, HairRoot, RuntimeCommand, Simulation,
    SimulationConfig, Vec3,
    debris::{DebrisPoint, DebrisSegment},
    init::{build_strands, generate_hair_roots},
    physics::{update_debris, update_strands},
};

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0e-4
}

#[test]
fn simulation_new_default_succeeds() {
    let sim = Simulation::new(SimulationConfig::default());
    assert!(sim.is_ok());
}

#[test]
fn config_validation_rejects_invalid_values() {
    let mut cfg = SimulationConfig::default();
    cfg.head.radii.x = 0.0;
    assert!(cfg.validate().is_err());

    let mut cfg = SimulationConfig::default();
    cfg.tuning.nodes_per_strand = 0;
    assert!(cfg.validate().is_err());

    let mut cfg = SimulationConfig::default();
    cfg.tuning.clipper_radius = -0.1;
    assert!(cfg.validate().is_err());

    let mut cfg = SimulationConfig::default();
    cfg.tuning.hair_length = f32::NAN;
    assert!(cfg.validate().is_err());
}

#[test]
fn hair_generation_returns_requested_number_of_roots() {
    let cfg = SimulationConfig::default();
    let head = Ellipsoid::new(cfg.head.center, cfg.head.radii);
    let mut rng = StdRng::seed_from_u64(7);

    let roots = generate_hair_roots(&head, 128, &mut rng);
    assert_eq!(roots.len(), 128);
}

#[test]
fn generated_roots_obey_face_mask_rule() {
    let cfg = SimulationConfig::default();
    let head = Ellipsoid::new(cfg.head.center, cfg.head.radii);
    let mut rng = StdRng::seed_from_u64(9);

    let roots = generate_hair_roots(&head, 256, &mut rng);
    assert!(
        roots
            .iter()
            .all(|root| !(root.normal.y < -0.4 && root.normal.z < 0.8))
    );
}

#[test]
fn resolve_sphere_contact_pushes_internal_point_outside_head() {
    let head = Ellipsoid::new(Vec3::new(0.0, 0.0, 0.8), Vec3::new(0.75, 0.85, 0.85));
    let resolved = head.resolve_sphere_contact(Vec3::new(0.0, 0.0, 0.8), 0.12);

    assert!(head.normalized_distance_squared(resolved) > 1.0);
}

#[test]
fn set_hair_length_rebuilds_geometry_and_preserves_valid_active_lengths() {
    let mut sim = Simulation::new(SimulationConfig::default()).unwrap();
    sim.set_hair_length(1.5).unwrap();

    let strand = &sim.strands()[0];
    assert_eq!(strand.active_len, sim.config().tuning.nodes_per_strand);
    assert!(approx_eq(
        strand.active_nodes()[0]
            .position
            .distance(strand.active_nodes().last().unwrap().position),
        1.5
    ));
    assert!(
        strand
            .active_nodes()
            .iter()
            .all(|node| node.velocity == Vec3::ZERO)
    );
}

#[test]
fn set_density_factor_changes_active_strand_count() {
    let mut sim = Simulation::new(SimulationConfig::default()).unwrap();
    sim.set_density_factor(0.5).unwrap();
    assert_eq!(sim.strands().len(), 325);

    sim.set_density_factor(1.5).unwrap();
    assert_eq!(sim.strands().len(), 975);
}

#[test]
fn runtime_setters_reject_non_finite_values() {
    let mut sim = Simulation::new(SimulationConfig::default()).unwrap();

    assert!(sim.set_rigidity(f32::NAN).is_err());
    assert!(sim.set_hair_length(f32::NAN).is_err());
    assert!(sim.set_density_factor(f32::NAN).is_err());
    assert!(
        sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::SetTargetXz {
            x: f32::NAN,
            z: 1.0,
        }))
        .is_err()
    );
}

#[test]
fn cutting_a_strand_produces_debris() {
    let head_center = Vec3::new(0.0, 0.0, 0.8);
    let root = HairRoot {
        offset: Vec3::ZERO,
        normal: Vec3::new(0.0, 1.0, 0.0),
    };
    let mut strands = build_strands(vec![root], 6, head_center, 1.0);
    let cut_position = strands[0].nodes[2].position;
    let clipper = ClipperState {
        initial_pos: cut_position,
        target_pos: cut_position,
        actual_pos: cut_position,
        radius: 0.2,
        is_cutting: true,
    };
    let mut debris = Vec::new();
    let mut rng = StdRng::seed_from_u64(11);

    update_strands(
        &mut strands,
        &mut debris,
        &clipper,
        head_center,
        Vec3::new(0.0, 0.0, -0.04),
        0.95,
        0.08,
        1.0,
        6,
        0.8,
        &mut rng,
    );

    assert!(!debris.is_empty());
    assert!(strands[0].active_len < 6);
}

#[test]
fn debris_stops_at_floor_after_updates() {
    let mut debris = vec![DebrisSegment::new(vec![DebrisPoint {
        position: Vec3::new(0.0, 0.0, 0.0),
        velocity: Vec3::ZERO,
        is_stopped: false,
    }])];

    for _ in 0..50 {
        update_debris(&mut debris, Vec3::new(0.0, 0.0, -0.04), -1.5, 0.8);
    }

    let point = &debris[0].points[0];
    assert!(point.is_stopped);
    assert!(approx_eq(point.position.z, -1.5));
    assert_eq!(point.velocity, Vec3::ZERO);
}

#[test]
fn smaller_dt_produces_smaller_single_step_motion() {
    let head_center = Vec3::new(0.0, 0.0, 0.8);
    let root = HairRoot {
        offset: Vec3::ZERO,
        normal: Vec3::new(1.0, 0.0, 0.0),
    };
    let clipper = ClipperState::new(Vec3::new(5.0, 5.0, 5.0), 0.1);
    let gravity = Vec3::new(0.0, 0.0, -0.04);
    let mut rng_small = StdRng::seed_from_u64(21);
    let mut rng_large = StdRng::seed_from_u64(21);
    let mut strands_small = build_strands(vec![root.clone()], 6, head_center, 1.0);
    let mut strands_large = build_strands(vec![root], 6, head_center, 1.0);

    update_strands(
        &mut strands_small,
        &mut Vec::new(),
        &clipper,
        head_center,
        gravity,
        0.95,
        0.08,
        1.0,
        6,
        0.1,
        &mut rng_small,
    );
    update_strands(
        &mut strands_large,
        &mut Vec::new(),
        &clipper,
        head_center,
        gravity,
        0.95,
        0.08,
        1.0,
        6,
        1.0,
        &mut rng_large,
    );

    let small_tip_delta = strands_small[0].nodes[5]
        .position
        .distance(head_center + strands_small[0].root.offset + Vec3::new(1.0, 0.0, 0.0));
    let large_tip_delta = strands_large[0].nodes[5]
        .position
        .distance(head_center + strands_large[0].root.offset + Vec3::new(1.0, 0.0, 0.0));
    assert!(large_tip_delta > small_tip_delta);
}

#[test]
fn debris_update_respects_dt() {
    let gravity = Vec3::new(0.0, 0.0, -0.04);
    let mut fast = DebrisSegment::new(vec![DebrisPoint {
        position: Vec3::new(0.0, 0.0, 0.0),
        velocity: Vec3::ZERO,
        is_stopped: false,
    }]);
    let mut slow = fast.clone();

    fast.update(gravity, -10.0, 1.0);
    slow.update(gravity, -10.0, 0.25);

    assert!(fast.points[0].position.z < slow.points[0].position.z);
}

#[test]
fn fixed_seed_produces_deterministic_initial_roots() {
    let cfg = SimulationConfig::default();
    let head = Ellipsoid::new(cfg.head.center, cfg.head.radii);
    let mut rng_a = StdRng::seed_from_u64(12345);
    let mut rng_b = StdRng::seed_from_u64(12345);

    let roots_a = generate_hair_roots(&head, 32, &mut rng_a);
    let roots_b = generate_hair_roots(&head, 32, &mut rng_b);

    assert_eq!(roots_a.len(), roots_b.len());
    assert!(
        roots_a
            .iter()
            .zip(roots_b.iter())
            .all(|(a, b)| { a.offset == b.offset && a.normal == b.normal })
    );
}

#[test]
fn simulation_smoke_test_runs_without_panic_and_keeps_state_valid() {
    let mut sim = Simulation::new(SimulationConfig::default()).unwrap();
    sim.apply_command(RuntimeCommand::Clipper(ClipperCommand::ActivateCutting))
        .unwrap();

    for _ in 0..8 {
        sim.apply_commands(&[
            RuntimeCommand::Clipper(ClipperCommand::MoveRight),
            RuntimeCommand::Clipper(ClipperCommand::MoveForward),
        ])
        .unwrap();
        sim.step();
    }

    assert!(!sim.strands().is_empty());
    assert!(
        sim.strands()
            .iter()
            .all(|strand| strand.active_len >= 1 && strand.active_len <= strand.nodes.len())
    );
    assert!(
        sim.debris()
            .iter()
            .flat_map(|segment| segment.points.iter())
            .all(|point| point.position.z >= sim.config().bounds.floor_z)
    );
}

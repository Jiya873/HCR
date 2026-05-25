use rand::{SeedableRng, rngs::StdRng};

use crate::{
    clipper::{ClipperCommand, ClipperState},
    commands::{RuntimeCommand, clipper_delta},
    config::SimulationConfig,
    debris::DebrisSegment,
    geometry::Ellipsoid,
    hair::HairStrand,
    init::{build_strands, generate_hair_roots},
    physics::{update_debris, update_strands},
};

#[derive(Clone, Debug)]
pub struct StepSummary {
    pub strands_cut_this_step: usize,
    pub debris_segments_created: usize,
}

#[derive(Clone, Debug)]
pub struct SimulationSnapshot {
    pub clipper: ClipperState,
    pub strands: Vec<HairStrand>,
    pub debris: Vec<DebrisSegment>,
}

pub struct Simulation {
    config: SimulationConfig,
    head: Ellipsoid,
    clipper: ClipperState,
    strands: Vec<HairStrand>,
    debris: Vec<DebrisSegment>,
    rng: StdRng,
    active_strand_count: usize,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Result<Self, String> {
        config.validate()?;

        let head = Ellipsoid::new(config.head.center, config.head.radii);
        let mut rng = StdRng::seed_from_u64(config.rng_seed);
        let max_strands = config.tuning.base_num_strands.saturating_mul(2);
        let roots = generate_hair_roots(&head, max_strands, &mut rng);
        let strands = build_strands(
            roots,
            config.tuning.nodes_per_strand,
            config.head.center,
            config.tuning.hair_length,
        );
        let active_strand_count = active_strand_count(&config, strands.len());

        let mut clipper =
            ClipperState::new(config.clipper_initial_pos, config.tuning.clipper_radius);
        clipper.resolve_against_head(&head);

        Ok(Self {
            config,
            head,
            clipper,
            strands,
            debris: Vec::new(),
            rng,
            active_strand_count,
        })
    }

    pub fn step(&mut self) -> StepSummary {
        self.clipper.resolve_against_head(&self.head);

        let before_lengths = self
            .strands()
            .iter()
            .map(|strand| strand.active_len)
            .collect::<Vec<_>>();
        let debris_before = self.debris.len();

        let active_count = self.active_strand_count.min(self.strands.len());
        update_strands(
            &mut self.strands[..active_count],
            &mut self.debris,
            &self.clipper,
            self.config.head.center,
            self.config.tuning.gravity,
            self.config.tuning.damping,
            self.config.tuning.rigidity,
            self.config.tuning.hair_length,
            self.config.tuning.nodes_per_strand,
            self.config.tuning.dt,
            &mut self.rng,
        );
        update_debris(
            &mut self.debris,
            self.config.tuning.gravity,
            self.config.bounds.floor_z,
            self.config.tuning.dt,
        );

        let strands_cut_this_step = self
            .strands()
            .iter()
            .zip(before_lengths.iter())
            .filter(|(strand, before)| strand.active_len < **before)
            .count();

        StepSummary {
            strands_cut_this_step,
            debris_segments_created: self.debris.len().saturating_sub(debris_before),
        }
    }

    pub fn apply_command(&mut self, command: RuntimeCommand) -> Result<(), String> {
        match command {
            RuntimeCommand::Clipper(cmd) => self.apply_clipper_command(cmd),
            RuntimeCommand::SetRigidity(value) => self.set_rigidity(value),
            RuntimeCommand::SetHairLength(value) => self.set_hair_length(value),
            RuntimeCommand::SetDensityFactor(value) => self.set_density_factor(value),
            RuntimeCommand::Regenerate { seed } => {
                self.regenerate(seed);
                Ok(())
            }
        }
    }

    pub fn apply_commands(&mut self, commands: &[RuntimeCommand]) -> Result<(), String> {
        for command in commands {
            self.apply_command(command.clone())?;
        }
        Ok(())
    }

    pub fn reset_clipper(&mut self) {
        self.clipper.reset();
        self.clipper.resolve_against_head(&self.head);
    }

    pub fn regenerate(&mut self, seed: u64) {
        self.config.rng_seed = seed;
        self.rng = StdRng::seed_from_u64(seed);
        let max_strands = self.config.tuning.base_num_strands.saturating_mul(2);
        let roots = generate_hair_roots(&self.head, max_strands, &mut self.rng);
        self.strands = build_strands(
            roots,
            self.config.tuning.nodes_per_strand,
            self.config.head.center,
            self.config.tuning.hair_length,
        );
        self.active_strand_count = active_strand_count(&self.config, self.strands.len());
        self.debris.clear();
        self.clipper.resolve_against_head(&self.head);
    }

    pub fn rebuild_hair_geometry(&mut self) {
        for strand in &mut self.strands {
            strand.reset_geometry(self.config.head.center, self.config.tuning.hair_length);
        }
        self.active_strand_count = active_strand_count(&self.config, self.strands.len());
    }

    pub fn set_rigidity(&mut self, rigidity: f32) -> Result<(), String> {
        if !rigidity.is_finite() {
            return Err("rigidity must be finite".to_string());
        }
        if rigidity < 0.0 {
            return Err("rigidity must be non-negative".to_string());
        }
        self.config.tuning.rigidity = rigidity;
        Ok(())
    }

    pub fn set_hair_length(&mut self, hair_length: f32) -> Result<(), String> {
        if !hair_length.is_finite() {
            return Err("hair_length must be finite".to_string());
        }
        if hair_length <= 0.0 {
            return Err("hair_length must be positive".to_string());
        }
        self.config.tuning.hair_length = hair_length;
        self.rebuild_hair_geometry();
        Ok(())
    }

    pub fn set_density_factor(&mut self, density_factor: f32) -> Result<(), String> {
        if !density_factor.is_finite() {
            return Err("density_factor must be finite".to_string());
        }
        if !(0.0 < density_factor && density_factor <= 2.0) {
            return Err("density_factor must be within (0, 2]".to_string());
        }
        self.config.tuning.density_factor = density_factor;
        self.rebuild_hair_geometry();
        Ok(())
    }

    pub fn clipper(&self) -> &ClipperState {
        &self.clipper
    }

    pub fn strands(&self) -> &[HairStrand] {
        &self.strands[..self.active_strand_count.min(self.strands.len())]
    }

    pub fn debris(&self) -> &[DebrisSegment] {
        &self.debris
    }

    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }

    pub fn snapshot(&self) -> SimulationSnapshot {
        SimulationSnapshot {
            clipper: self.clipper.clone(),
            strands: self.strands().to_vec(),
            debris: self.debris.clone(),
        }
    }

    fn apply_clipper_command(&mut self, cmd: ClipperCommand) -> Result<(), String> {
        if let Some(delta) = clipper_delta(&cmd, self.config.tuning.move_speed) {
            self.clipper
                .move_by(delta, self.config.bounds.min, self.config.bounds.max);
        } else {
            match cmd {
                ClipperCommand::SetTargetXz { x, z } => {
                    if !x.is_finite() || !z.is_finite() {
                        return Err("clipper target coordinates must be finite".to_string());
                    }
                    self.clipper
                        .set_target_xz(x, z, self.config.bounds.min, self.config.bounds.max)
                }
                ClipperCommand::Reset => self.reset_clipper(),
                ClipperCommand::ActivateCutting => self.clipper.set_cutting(true),
                ClipperCommand::DeactivateCutting => self.clipper.set_cutting(false),
                ClipperCommand::MoveUp
                | ClipperCommand::MoveDown
                | ClipperCommand::MoveLeft
                | ClipperCommand::MoveRight
                | ClipperCommand::MoveForward
                | ClipperCommand::MoveBackward => unreachable!(),
            }
        }

        self.clipper.resolve_against_head(&self.head);
        Ok(())
    }
}

fn active_strand_count(config: &SimulationConfig, total: usize) -> usize {
    let requested = (config.tuning.base_num_strands as f32 * config.tuning.density_factor).round();
    (requested.max(0.0) as usize).min(total)
}

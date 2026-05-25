use rand::Rng;

use crate::{
    clipper::ClipperState,
    debris::{DebrisPoint, DebrisSegment},
    hair::HairStrand,
    math::Vec3,
};

pub fn update_strands(
    strands: &mut [HairStrand],
    debris_out: &mut Vec<DebrisSegment>,
    clipper: &ClipperState,
    head_center: Vec3,
    gravity: Vec3,
    damping: f32,
    rigidity: f32,
    hair_length: f32,
    nodes_per_strand: usize,
    dt: f32,
    rng: &mut impl Rng,
) {
    let target_segment_length = if nodes_per_strand <= 1 {
        0.0
    } else {
        hair_length / (nodes_per_strand - 1) as f32
    };
    let cut_radius_sq = (clipper.radius * 0.95).powi(2);
    let stiffness_denom = (nodes_per_strand.saturating_sub(1)).max(1) as f32;

    for strand in strands {
        if strand.nodes.is_empty() {
            continue;
        }

        let root_position = head_center + strand.root.offset;
        strand.nodes[0].position = root_position;
        strand.nodes[0].velocity = Vec3::ZERO;

        let active_len = strand.active_len.min(strand.nodes.len());
        if active_len < 2 {
            continue;
        }

        let normal = strand.root.normal.normalized();
        let mut cut_idx = None;

        for index in 1..active_len {
            let (before, after) = strand.nodes.split_at_mut(index);
            let prev_pos = before[index - 1].position;
            let node = &mut after[0];

            if clipper.is_cutting
                && node.position.distance_squared(clipper.actual_pos) < cut_radius_sq
            {
                cut_idx = Some(index);
                break;
            }

            let stiffness = rigidity * (1.0 - index as f32 / stiffness_denom);
            node.velocity += gravity * dt;
            node.velocity += normal * (stiffness * dt);
            node.position += node.velocity * dt;
            node.velocity *= damping;

            let curr_vec = node.position - prev_pos;
            let curr_len = curr_vec.length();
            if curr_len > 1.0e-6 {
                let correction = ((curr_len - target_segment_length) / curr_len) * 0.85;
                node.position -= curr_vec * correction;
            }
        }

        if let Some(cut_idx) = cut_idx {
            let tail_points = strand.nodes[cut_idx..active_len]
                .iter()
                .map(|node| DebrisPoint {
                    position: node.position,
                    velocity: Vec3::new(
                        rng.gen_range(-0.025..0.025),
                        rng.gen_range(-0.025..0.025),
                        gravity.z * 0.5,
                    ),
                    is_stopped: false,
                })
                .collect::<Vec<_>>();

            if !tail_points.is_empty() {
                debris_out.push(DebrisSegment::new(tail_points));
            }

            strand.shorten_to(cut_idx);
        }
    }
}

pub fn update_debris(debris: &mut [DebrisSegment], gravity: Vec3, floor_z: f32, dt: f32) {
    for segment in debris {
        segment.update(gravity, floor_z, dt);
    }
}

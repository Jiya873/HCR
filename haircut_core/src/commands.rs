use crate::{clipper::ClipperCommand, math::Vec3};

#[derive(Clone, Debug)]
pub enum RuntimeCommand {
    Clipper(ClipperCommand),
    SetRigidity(f32),
    SetHairLength(f32),
    SetDensityFactor(f32),
    Regenerate { seed: u64 },
}

pub fn clipper_delta(cmd: &ClipperCommand, move_speed: f32) -> Option<Vec3> {
    match cmd {
        ClipperCommand::MoveUp => Some(Vec3::new(0.0, 0.0, move_speed)),
        ClipperCommand::MoveDown => Some(Vec3::new(0.0, 0.0, -move_speed)),
        ClipperCommand::MoveLeft => Some(Vec3::new(-move_speed, 0.0, 0.0)),
        ClipperCommand::MoveRight => Some(Vec3::new(move_speed, 0.0, 0.0)),
        ClipperCommand::MoveForward => Some(Vec3::new(0.0, move_speed, 0.0)),
        ClipperCommand::MoveBackward => Some(Vec3::new(0.0, -move_speed, 0.0)),
        _ => None,
    }
}

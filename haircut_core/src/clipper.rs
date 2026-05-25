use crate::{geometry::Ellipsoid, math::Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClipperCommand {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveForward,
    MoveBackward,
    SetTargetXz { x: f32, z: f32 },
    Reset,
    ActivateCutting,
    DeactivateCutting,
}

#[derive(Clone, Debug)]
pub struct ClipperState {
    pub initial_pos: Vec3,
    pub target_pos: Vec3,
    pub actual_pos: Vec3,
    pub radius: f32,
    pub is_cutting: bool,
}

impl ClipperState {
    pub fn new(initial_pos: Vec3, radius: f32) -> Self {
        Self {
            initial_pos,
            target_pos: initial_pos,
            actual_pos: initial_pos,
            radius,
            is_cutting: false,
        }
    }

    pub fn reset(&mut self) {
        self.target_pos = self.initial_pos;
        self.actual_pos = self.initial_pos;
    }

    pub fn set_cutting(&mut self, enabled: bool) {
        self.is_cutting = enabled;
    }

    pub fn move_by(&mut self, delta: Vec3, bounds_min: Vec3, bounds_max: Vec3) {
        self.target_pos = (self.target_pos + delta).clamp(bounds_min, bounds_max);
        self.actual_pos = self.target_pos;
    }

    pub fn set_target_xz(&mut self, x: f32, z: f32, bounds_min: Vec3, bounds_max: Vec3) {
        self.target_pos.x = x;
        self.target_pos.z = z;
        self.target_pos = self.target_pos.clamp(bounds_min, bounds_max);
        self.actual_pos = self.target_pos;
    }

    pub fn resolve_against_head(&mut self, head: &Ellipsoid) {
        self.actual_pos = head.resolve_sphere_contact(self.target_pos, self.radius);
    }
}

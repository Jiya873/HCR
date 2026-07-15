use crate::{geometry::Ellipsoid, math::Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClipperCommand {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveForward,
    MoveBackward,
    SetTargetXyz { x: f32, y: f32, z: f32 },
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

    pub fn set_target_xyz(&mut self, x: f32, y: f32, z: f32, bounds_min: Vec3, bounds_max: Vec3) {
        self.target_pos.x = x;
        self.target_pos.y = y; // <-- NOW WE ARE UPDATING HEIGHT
        self.target_pos.z = z;
        self.target_pos = self.target_pos.clamp(bounds_min, bounds_max);
        self.actual_pos = self.target_pos;
    }

    pub fn resolve_against_head(&mut self, head: &Ellipsoid) {
        self.actual_pos = head.resolve_sphere_contact(self.target_pos, self.radius);
    }

    pub fn update_kinematics(&mut self, speed: f32, dt: f32) {
        let dx = self.target_pos.x - self.actual_pos.x;
        let dy = self.target_pos.y - self.actual_pos.y;
        let dz = self.target_pos.z - self.actual_pos.z;

        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        if distance < 0.0001 {
            self.actual_pos.x = self.target_pos.x;
            self.actual_pos.y = self.target_pos.y;
            self.actual_pos.z = self.target_pos.z;
        } else {
            let max_step = speed * dt;

            if max_step >= distance {
                self.actual_pos.x = self.target_pos.x;
                self.actual_pos.y = self.target_pos.y;
                self.actual_pos.z = self.target_pos.z;
            } else {
                let ratio = max_step / distance;
                self.actual_pos.x += dx * ratio;
                self.actual_pos.y += dy * ratio;
                self.actual_pos.z += dz * ratio;
            }
        }
    }
}

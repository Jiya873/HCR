use crate::math::Vec3;

#[derive(Clone, Debug)]
pub struct DebrisPoint {
    pub position: Vec3,
    pub velocity: Vec3,
    pub is_stopped: bool,
}

#[derive(Clone, Debug)]
pub struct DebrisSegment {
    pub points: Vec<DebrisPoint>,
}

impl DebrisSegment {
    pub fn new(points: Vec<DebrisPoint>) -> Self {
        Self { points }
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn update(&mut self, gravity: Vec3, floor_y: f32, dt: f32) {
        for point in &mut self.points {
            if point.is_stopped {
                continue;
            }

            point.velocity += gravity * (1.5 * dt);
            point.position += point.velocity * dt;

            if point.position.y <= floor_y {
                point.position.y = floor_y;
                point.velocity = Vec3::ZERO;
                point.is_stopped = true;
            }
        }
    }
}

use crate::math::Vec3;

/// Head ellipsoid: where the head sits and its three semi-axis radii.
#[derive(Clone, Debug)]
pub struct HeadConfig {
    pub center: Vec3,
    pub radii: Vec3,
}

/// Clipper clamp box (`min`/`max`) and the floor plane at which debris comes to rest.
///
/// `min` and `max` are the corner `Vec3`s of an axis-aligned box used only to
/// clamp the clipper's target position. `floor_z` is read only by the debris
/// simulation — it can sit below `min.z` so cut hair falls past the clipper's
/// reachable volume before stopping.
#[derive(Clone, Debug)]
pub struct WorldBounds {
    pub min: Vec3,
    pub max: Vec3,
    pub floor_y: f32,
}

/// All numeric tunables driving the integrator, hair, and clipper kinematics.
#[derive(Clone, Debug)]
pub struct TuningConfig {
    pub gravity: Vec3,
    pub damping: f32,
    pub rigidity: f32,
    pub hair_length: f32,
    pub density_factor: f32,
    pub nodes_per_strand: usize,
    pub base_num_strands: usize,
    pub clipper_radius: f32,
    pub move_speed: f32,
    pub dt: f32,
}

/// Top-level configuration: head, bounds, tuning, plus clipper start and RNG seed.
#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub head: HeadConfig,
    pub bounds: WorldBounds,
    pub tuning: TuningConfig,
    pub clipper_initial_pos: Vec3,
    pub rng_seed: u64,
}

impl Default for HeadConfig {
    fn default() -> Self {
        Self {
            center: Vec3::new(0.0, 0.3, 0.7),
            radii: Vec3::new(0.75, 0.85, 0.85),
        }
    }
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            min: Vec3::new(-2.0, -2.0, -0.5),
            max: Vec3::new(2.0, 2.0, 3.0),
            floor_y: -0.6,
        }
    }
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -0.04, 0.0),
            damping: 0.95,
            rigidity: 0.08,
            hair_length: 1.0,
            density_factor: 1.0,
            nodes_per_strand: 12,
            base_num_strands: 1650,
            clipper_radius: 0.12,
            move_speed: 0.04,
            dt: 0.2,
        }
    }
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            head: HeadConfig::default(),
            bounds: WorldBounds::default(),
            tuning: TuningConfig::default(),
            clipper_initial_pos: Vec3::new(0.0, 2.0, -1.7),
            rng_seed: 1,
        }
    }
}

impl SimulationConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !is_finite_vec3(self.head.center) {
            return Err("head center must be finite".to_string());
        }

        let radii = self.head.radii;
        if !is_finite_vec3(radii) {
            return Err("head radii must be finite".to_string());
        }
        if radii.x <= 0.0 || radii.y <= 0.0 || radii.z <= 0.0 {
            return Err("head radii must be positive".to_string());
        }

        if !is_finite_vec3(self.bounds.min) || !is_finite_vec3(self.bounds.max) {
            return Err("world bounds must be finite".to_string());
        }
        if !self.bounds.floor_y.is_finite() {
            return Err("floor_y must be finite".to_string());
        }

        if !is_finite_vec3(self.tuning.gravity) {
            return Err("gravity must be finite".to_string());
        }
        if self.tuning.nodes_per_strand < 2 {
            return Err("nodes_per_strand must be at least 2".to_string());
        }
        if self.tuning.base_num_strands == 0 {
            return Err("base_num_strands must be positive".to_string());
        }
        if !self.tuning.clipper_radius.is_finite() {
            return Err("clipper_radius must be finite".to_string());
        }
        if self.tuning.clipper_radius <= 0.0 {
            return Err("clipper_radius must be positive".to_string());
        }
        if !self.tuning.damping.is_finite() {
            return Err("damping must be finite".to_string());
        }
        if self.tuning.damping < 0.0 || self.tuning.damping > 1.0 {
            return Err("damping must be within [0, 1]".to_string());
        }
        if !self.tuning.rigidity.is_finite() {
            return Err("rigidity must be finite".to_string());
        }
        if self.tuning.rigidity < 0.0 {
            return Err("rigidity must be non-negative".to_string());
        }
        if !self.tuning.hair_length.is_finite() {
            return Err("hair_length must be finite".to_string());
        }
        if self.tuning.hair_length <= 0.0 {
            return Err("hair_length must be positive".to_string());
        }
        if !self.tuning.density_factor.is_finite() {
            return Err("density_factor must be finite".to_string());
        }
        if self.tuning.density_factor <= 0.0 || self.tuning.density_factor > 2.0 {
            return Err("density_factor must be within (0, 2]".to_string());
        }
        if !self.tuning.move_speed.is_finite() {
            return Err("move_speed must be finite".to_string());
        }
        if self.tuning.move_speed <= 0.0 {
            return Err("move_speed must be positive".to_string());
        }
        if !self.tuning.dt.is_finite() {
            return Err("dt must be finite".to_string());
        }
        if self.tuning.dt <= 0.0 {
            return Err("dt must be positive".to_string());
        }
        if !is_finite_vec3(self.clipper_initial_pos) {
            return Err("clipper_initial_pos must be finite".to_string());
        }
        if self.bounds.min.x > self.bounds.max.x
            || self.bounds.min.y > self.bounds.max.y
            || self.bounds.min.z > self.bounds.max.z
        {
            return Err("world bounds min must not exceed max".to_string());
        }
        if self.bounds.floor_y > self.bounds.max.y {
            return Err("floor_y must be at or below bounds.max.y".to_string());
        }

        Ok(())
    }
}

fn is_finite_vec3(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

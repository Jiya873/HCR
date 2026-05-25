use crate::math::Vec3;

/// Axis-aligned ellipsoid `((x−cx)/rx)² + ((y−cy)/ry)² + ((z−cz)/rz)² = 1`,
/// used as the head collider for the clipper.
#[derive(Clone, Copy, Debug)]
pub struct Ellipsoid {
    pub center: Vec3,
    pub radii: Vec3,
}

impl Ellipsoid {
    pub fn new(center: Vec3, radii: Vec3) -> Self {
        Self { center, radii }
    }

    /// Implicit-form value: `<1` inside, `=1` on the surface, `>1` outside.
    pub fn normalized_distance_squared(&self, p: Vec3) -> f32 {
        let rel = p - self.center;
        let rx = self.radii.x.max(1.0e-6);
        let ry = self.radii.y.max(1.0e-6);
        let rz = self.radii.z.max(1.0e-6);

        (rel.x / rx).powi(2) + (rel.y / ry).powi(2) + (rel.z / rz).powi(2)
    }

    pub fn contains(&self, p: Vec3) -> bool {
        self.normalized_distance_squared(p) < 1.0
    }

    /// Ray-from-center projection of `p` onto the surface (exact for spheres,
    /// approximate for ellipsoids — not the true closest-point projection).
    pub fn project_to_surface(&self, p: Vec3) -> Vec3 {
        let rel = p - self.center;
        let norm_sq = self.normalized_distance_squared(p);
        if norm_sq <= 1.0e-8 {
            return self.center + Vec3::new(0.0, 0.0, self.radii.z);
        }

        let factor = 1.0 / norm_sq.sqrt();
        self.center + rel * factor
    }

    /// Outward unit normal at a surface point, taken from the implicit-function gradient.
    pub fn surface_normal(&self, p_on_surface: Vec3) -> Vec3 {
        let rel = p_on_surface - self.center;
        let gradient = Vec3::new(
            rel.x / self.radii.x.max(1.0e-6).powi(2),
            rel.y / self.radii.y.max(1.0e-6).powi(2),
            rel.z / self.radii.z.max(1.0e-6).powi(2),
        );
        gradient.normalized()
    }

    /// The module's only entry point used by the rest of the crate: if the
    /// sphere overlaps the ellipsoid, return its center pushed out to surface
    /// + `sphere_radius` along the normal; otherwise return it unchanged.
    pub fn resolve_sphere_contact(&self, sphere_center: Vec3, sphere_radius: f32) -> Vec3 {
        if self.contains(sphere_center) {
            let surface = self.project_to_surface(sphere_center);
            let normal = self.surface_normal(surface);
            surface + normal * sphere_radius.max(0.0)
        } else {
            sphere_center
        }
    }
}

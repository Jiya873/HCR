use rand::Rng;

use crate::{
    geometry::Ellipsoid,
    hair::{HairRoot, HairStrand},
    math::Vec3,
};

pub fn generate_hair_roots(
    head: &Ellipsoid,
    max_strands: usize,
    rng: &mut impl Rng,
) -> Vec<HairRoot> {
    let mut roots = Vec::with_capacity(max_strands);

    while roots.len() < max_strands {
        let theta = rng.gen_range(0.0..(core::f32::consts::PI * 2.0));
        let phi = rng.gen_range(0.0..((core::f32::consts::PI / 2.0) * 0.9));

        let ux = theta.cos() * phi.cos();
        let uy = theta.sin() * phi.cos();
        let uz = phi.sin();

        if uy < -0.4 && uz < 0.8 {
            continue;
        }

        let offset = Vec3::new(ux * head.radii.x, uy * head.radii.y, uz * head.radii.z);
        let normal = Vec3::new(ux, uy, uz).normalized();
        roots.push(HairRoot { offset, normal });
    }

    roots
}

pub fn build_strands(
    roots: Vec<HairRoot>,
    nodes_per_strand: usize,
    head_center: Vec3,
    hair_length: f32,
) -> Vec<HairStrand> {
    roots
        .into_iter()
        .map(|root| {
            let mut strand = HairStrand::new(root, nodes_per_strand);
            strand.reset_geometry(head_center, hair_length);
            strand
        })
        .collect()
}

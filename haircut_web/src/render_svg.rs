use std::fmt::Write;

use haircut_core::{Simulation, Vec3};

const AZ_DEG: f32 = -20.0;
const EL_DEG: f32 = 15.0;
const VIEW_CENTER: Vec3 = Vec3::new(0.0, 0.0, 0.5);

fn project(p: Vec3) -> (f32, f32) {
    let px = p.x - VIEW_CENTER.x;
    let py = p.y - VIEW_CENTER.y;
    let pz = p.z - VIEW_CENTER.z;
    let (sa, ca) = AZ_DEG.to_radians().sin_cos();
    let (se, ce) = EL_DEG.to_radians().sin_cos();
    let sx = px * ca + py * sa;
    let sy = px * (-sa * se) + py * (ca * se) + pz * ce;
    (sx, -sy)
}

pub fn render(sim: &Simulation, tick: u64) -> String {
    let snap = sim.snapshot();
    let cfg = sim.config();
    let mut s = String::with_capacity(64 * 1024);

    let _ = write!(
        s,
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="-3 -3 6 6" preserveAspectRatio="xMidYMid meet" style="background:#f8f8fa">"##
    );

    let floor_z = cfg.bounds.floor_z;
    let _ = write!(s, r##"<g stroke="#c8c8cc" stroke-width="0.008" fill="none">"##);
    let (g_min, g_max, g_step) = (-2.0_f32, 2.0_f32, 0.5_f32);
    let mut g = g_min;
    while g <= g_max + 1e-3 {
        let (x1, y1) = project(Vec3::new(g_min, g, floor_z));
        let (x2, y2) = project(Vec3::new(g_max, g, floor_z));
        let _ = write!(s, r#"<line x1="{x1:.3}" y1="{y1:.3}" x2="{x2:.3}" y2="{y2:.3}"/>"#);
        let (x1, y1) = project(Vec3::new(g, g_min, floor_z));
        let (x2, y2) = project(Vec3::new(g, g_max, floor_z));
        let _ = write!(s, r#"<line x1="{x1:.3}" y1="{y1:.3}" x2="{x2:.3}" y2="{y2:.3}"/>"#);
        g += g_step;
    }
    let _ = write!(s, "</g>");

    let hc = cfg.head.center;
    let hr = cfg.head.radii;
    let _ = write!(s, r##"<g stroke="#c5a99a" stroke-width="0.012" fill="none">"##);
    let lats = 6;
    let lons = 12;
    let segs = 36;
    for i in 1..lats {
        let phi = (i as f32 / lats as f32) * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;
        let z = phi.sin() * hr.z;
        let r = phi.cos();
        let _ = write!(s, r#"<polyline points=""#);
        for k in 0..=segs {
            let th = (k as f32 / segs as f32) * std::f32::consts::TAU;
            let (x, y) = project(Vec3::new(
                hc.x + r * th.cos() * hr.x,
                hc.y + r * th.sin() * hr.y,
                hc.z + z,
            ));
            let _ = write!(s, "{x:.3},{y:.3} ");
        }
        let _ = write!(s, r#""/>"#);
    }
    for l in 0..lons {
        let th = (l as f32 / lons as f32) * std::f32::consts::TAU;
        let _ = write!(s, r#"<polyline points=""#);
        for k in 0..=segs {
            let phi =
                (k as f32 / segs as f32) * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;
            let (x, y) = project(Vec3::new(
                hc.x + phi.cos() * th.cos() * hr.x,
                hc.y + phi.cos() * th.sin() * hr.y,
                hc.z + phi.sin() * hr.z,
            ));
            let _ = write!(s, "{x:.3},{y:.3} ");
        }
        let _ = write!(s, r#""/>"#);
    }
    let _ = write!(s, "</g>");

    let _ = write!(s, r##"<g stroke="#262626" stroke-width="0.009" fill="none">"##);
    for strand in &snap.strands {
        let nodes = strand.active_nodes();
        if nodes.len() < 2 {
            continue;
        }
        let _ = write!(s, r#"<polyline points=""#);
        for n in nodes {
            let (x, y) = project(n.position);
            let _ = write!(s, "{x:.3},{y:.3} ");
        }
        let _ = write!(s, r#""/>"#);
    }
    let _ = write!(s, "</g>");

    if !snap.debris.is_empty() {
        let _ = write!(s, r##"<g stroke="#8a8a8a" stroke-width="0.007" fill="none">"##);
        for seg in &snap.debris {
            if seg.points.len() < 2 {
                continue;
            }
            let _ = write!(s, r#"<polyline points=""#);
            for p in &seg.points {
                let (x, y) = project(p.position);
                let _ = write!(s, "{x:.3},{y:.3} ");
            }
            let _ = write!(s, r#""/>"#);
        }
        let _ = write!(s, "</g>");
    }

    let (cx, cy) = project(snap.clipper.actual_pos);
    let (fill, stroke) = if snap.clipper.is_cutting {
        ("#5fd97a", "#1f9b4a")
    } else {
        ("#e76d6d", "#aa2828")
    };
    let r = cfg.tuning.clipper_radius;
    let _ = write!(
        s,
        r##"<circle cx="{cx:.3}" cy="{cy:.3}" r="{r:.3}" fill="{fill}" fill-opacity="0.65" stroke="{stroke}" stroke-width="0.014"/>"##
    );

    let total = snap.strands.len();
    let cut = snap
        .strands
        .iter()
        .filter(|st| st.active_len < cfg.tuning.nodes_per_strand)
        .count();
    let intact = total.saturating_sub(cut);
    let debris = snap.debris.len();
    let cutting = snap.clipper.is_cutting;
    let _ = write!(
        s,
        r##"<g font-family="ui-monospace, Menlo, monospace" font-size="0.13" fill="#555">"##
    );
    let _ = write!(s, r#"<text x="-2.92" y="-2.78">tick {tick}</text>"#);
    let _ = write!(
        s,
        r#"<text x="-2.92" y="-2.58">intact {intact} · cut {cut} · debris {debris}</text>"#
    );
    let _ = write!(s, r#"<text x="-2.92" y="-2.38">cutting {cutting}</text>"#);
    let _ = write!(s, "</g>");

    let _ = write!(s, "</svg>");
    s
}

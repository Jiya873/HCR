struct Node {
    position: vec3<f32>,
    velocity: vec3<f32>,
}

struct HairStrand {
    nodes: array<Node, 12>,
    normal: vec3<f32>,
    active_len: u32,
}

struct PhysicsUniforms {
    gravity: vec3<f32>,
    dt: f32,
    damping: f32,
    rigidity: f32,
    segment_length: f32,
    _padding: f32,
}

@group(0) @binding(0) var<storage, read_write> strands: array<HairStrand>;
@group(0) @binding(1) var<uniform> params: PhysicsUniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&strands)) { return; }

    var strand = strands[index];
    if (strand.active_len < 2u) { return; }

    let stiffness_denom = f32(11u);

    for (var i = 1u; i < strand.active_len; i = i + 1u) {
        var pos = strand.nodes[i].position;
        var vel = strand.nodes[i].velocity;

        let stiffness = params.rigidity * (1.0 - f32(i) / stiffness_denom);

        vel = vel + (params.gravity * params.dt);
        vel = vel + (strand.normal * stiffness * params.dt);
        pos = pos + (vel * params.dt);
        vel = vel * params.damping;

        let prev_pos = strand.nodes[i - 1u].position;
        let curr_vec = pos - prev_pos;
        let curr_len = length(curr_vec);
        if (curr_len > 1.0e-6) {
            let correction = ((curr_len - params.segment_length) / curr_len) * 0.85;
            pos = pos - (curr_vec * correction);
        }

        strand.nodes[i].position = pos;
        strand.nodes[i].velocity = vel;
    }

    strands[index] = strand;
}
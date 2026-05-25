use crate::math::Vec3;

#[derive(Clone, Debug)]
pub struct HairRoot {
    pub offset: Vec3,
    pub normal: Vec3,
}

#[derive(Clone, Debug)]
pub struct HairNode {
    pub position: Vec3,
    pub velocity: Vec3,
}

#[derive(Clone, Debug)]
pub struct HairStrand {
    pub root: HairRoot,
    pub nodes: Vec<HairNode>,
    pub active_len: usize,
}

impl HairStrand {
    pub fn new(root: HairRoot, nodes_per_strand: usize) -> Self {
        let nodes = vec![
            HairNode {
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
            };
            nodes_per_strand
        ];

        Self {
            root,
            nodes,
            active_len: nodes_per_strand,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active_len > 1
    }

    pub fn active_nodes(&self) -> &[HairNode] {
        &self.nodes[..self.active_len.min(self.nodes.len())]
    }

    pub fn active_nodes_mut(&mut self) -> &mut [HairNode] {
        let end = self.active_len.min(self.nodes.len());
        &mut self.nodes[..end]
    }

    pub fn reset_geometry(&mut self, head_center: Vec3, hair_length: f32) {
        let root_position = head_center + self.root.offset;
        let direction = self.root.normal.normalized();
        let segment_length = self.segment_target_length(hair_length);

        self.active_len = self.nodes.len();

        for (index, node) in self.nodes.iter_mut().enumerate() {
            let dist = segment_length * index as f32;
            node.position = root_position + direction * dist;
            node.velocity = Vec3::ZERO;
        }
    }

    pub fn segment_target_length(&self, hair_length: f32) -> f32 {
        if self.nodes.len() <= 1 {
            0.0
        } else {
            hair_length / (self.nodes.len() - 1) as f32
        }
    }

    pub fn shorten_to(&mut self, new_active_len: usize) {
        self.active_len = new_active_len.clamp(1, self.nodes.len());
    }
}

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
}

pub const QUAD_VERTICES: &[QuadVertex] = &[
    // triangle 1
    QuadVertex {
        position: [-0.5, -0.5],
    },
    QuadVertex {
        position: [0.5, -0.5],
    },
    QuadVertex {
        position: [0.5, 0.5],
    },
    // triangle 2
    QuadVertex {
        position: [-0.5, -0.5],
    },
    QuadVertex {
        position: [0.5, 0.5],
    },
    QuadVertex {
        position: [-0.5, 0.5],
    },
];

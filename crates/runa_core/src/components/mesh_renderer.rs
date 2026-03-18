use runa_asset::TextureAsset;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub texture: Option<std::sync::Arc<TextureAsset>>,
}

impl Mesh {
    pub fn cube(size: f32) -> Self {
        let h = size * 0.5;

        // Вершины куба
        let vertices = vec![
            // Передняя грань
            Vertex3D {
                position: [-h, -h, h],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
            },
            Vertex3D {
                position: [h, -h, h],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 0.0],
            },
            Vertex3D {
                position: [h, h, h],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 1.0],
            },
            Vertex3D {
                position: [-h, h, h],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 1.0],
            },
            // ... остальные грани (6 граней × 4 вершины)
        ];

        // Индексы для треугольников
        let indices = vec![
            0, 1, 2, 2, 3,
            0, // передняя
              // ... остальные индексы
        ];

        Self {
            vertices,
            indices,
            texture: None,
        }
    }
}

#[derive(Clone)]
pub struct MeshRenderer {
    pub mesh: Mesh,
    pub color: [f32; 4],
}

impl MeshRenderer {
    pub fn new(mesh: Mesh) -> Self {
        Self {
            mesh,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

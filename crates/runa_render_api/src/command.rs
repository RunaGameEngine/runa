use std::sync::Arc;

use glam::USizeVec2;
use glam::{Mat4, Quat, Vec2, Vec3};

use runa_asset::TextureAsset;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub enum RenderCommands {
    Sprite {
        texture: std::sync::Arc<TextureAsset>,
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
    },
    Text {
        text: String,
        position: Vec2,
        color: [f32; 4],
        size: f32,
    },
    DebugRect {
        position: Vec3,
        size: Vec2,
        color: [f32; 4],
    },
    Tile {
        texture: Arc<TextureAsset>,
        position: Vec3,
        size: USizeVec2,
        uv_rect: [f32; 4],
        flip_x: bool,
        flip_y: bool,
        color: [f32; 4],
    },
    Mesh3D {
        vertices: Vec<Vertex3D>,
        indices: Vec<u32>,
        model_matrix: Mat4,
        color: [f32; 4],
    },
    // IU
    UiRect {
        rect: UiRect,
        color: [f32; 4],
        z_index: i16,
    },
    UiImage {
        texture: Arc<TextureAsset>,
        rect: UiRect,
        tint: [f32; 4],
        uv_rect: [f32; 4],
        z_index: i16,
    },
    UiText {
        text: String,
        rect: UiRect,
        color: [f32; 4],
        font_size: u16,
        z_index: i16,
    },
}

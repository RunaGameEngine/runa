use glam::{Quat, Vec2};
use glam::{USizeVec2, Vec3};

use runa_asset::TextureAsset;

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
        texture: std::sync::Arc<TextureAsset>,
        position: Vec3,    // мировая позиция левого-нижнего угла
        size: USizeVec2,   // размер тайла в мировых единицах
        uv_rect: [f32; 4], // [x, y, width, height] в текстурных координатах (0.0-1.0)
        flip_x: bool,
        flip_y: bool,
        color: [f32; 4], // RGBA тинт (1.0, 1.0, 1.0, 1.0 = без изменений)
    },
}

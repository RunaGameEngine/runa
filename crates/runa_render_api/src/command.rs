use glam::{Quat, Vec2};

use runa_asset::handle::Handle;
use runa_asset::texture::TextureAsset;

pub enum RenderCommands {
    Sprite {
        texture: Handle<TextureAsset>,
        position: Vec2,
        rotation: Quat,
        scale: Vec2,
    },
    Text {
        text: String,
        position: Vec2,
        color: [f32; 4],
        size: f32,
    },
    DebugRect {
        position: Vec2,
        size: Vec2,
        color: [f32; 4],
    },
}

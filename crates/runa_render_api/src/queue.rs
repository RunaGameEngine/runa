use crate::RenderCommands;
use glam::{Quat, USizeVec2, Vec2, Vec3};
use runa_asset::TextureAsset;

#[derive(Default)]
pub struct RenderQueue {
    pub commands: Vec<RenderCommands>,
}

impl RenderQueue {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push_sprite(
        &mut self,
        texture: std::sync::Arc<TextureAsset>,
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) {
        self.commands.push(RenderCommands::Sprite {
            texture,
            position,
            rotation,
            scale,
        });
    }

    pub fn draw_text(&mut self, text: String, position: Vec2, color: [f32; 4], size: f32) {
        self.commands.push(RenderCommands::Text {
            text,
            position,
            color,
            size,
        });
    }

    pub fn push_tile(
        &mut self,
        texture: std::sync::Arc<TextureAsset>,
        position: Vec3,
        size: USizeVec2,
        uv_rect: [f32; 4],
        flip_x: bool,
        flip_y: bool,
        color: [f32; 4],
    ) {
        self.commands.push(RenderCommands::Tile {
            texture,
            position,
            size,
            uv_rect,
            flip_x,
            flip_y,
            color,
        });
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

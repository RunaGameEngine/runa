use crate::command::RenderCommands;
use glam::{Quat, Vec2};
use runa_asset::{handle::Handle, texture::TextureAsset};

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

    pub fn draw_sprite(
        &mut self,
        texture: Handle<TextureAsset>,
        position: Vec2,
        rotation: Quat,
        scale: Vec2,
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

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

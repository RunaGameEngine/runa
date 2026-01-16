use glam::Mat4;

use crate::command::RenderCommands;
use runa_asset::{handle::Handle, texture::TextureAsset};

pub struct RenderQueue {
    pub commands: Vec<RenderCommands>,
}

impl RenderQueue {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn draw_sprite(&mut self, texture: Handle<TextureAsset>, model: Mat4) {
        self.commands
            .push(RenderCommands::Sprite { texture, model });
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

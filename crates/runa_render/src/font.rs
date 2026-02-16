use std::collections::HashMap;

use crate::resources::texture::GpuTexture;
use glam::Vec2;
use runa_asset::TextureAsset;
use runa_render_api::RenderCommands;
use wgpu::{Device, Queue};

pub struct FontManager {
    // Character texture cache
    textures: HashMap<char, GpuTexture>,
    // Character dimensions
    char_width: u32,
    char_height: u32,
    // Character spacing
    spacing: f32,
}

impl FontManager {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        // Create a basic 8x8 pixel font
        let mut font_manager = Self {
            textures: HashMap::new(),
            char_width: 8,
            char_height: 8,
            spacing: 1.0,
        };

        // Generate basic ASCII characters (space through tilde)
        for c in 32u8..127 {
            let ch = c as char;
            if !font_manager.textures.contains_key(&ch) {
                let texture = font_manager.generate_char_texture(device, queue, ch);
                font_manager.textures.insert(ch, texture);
            }
        }

        font_manager
    }

    fn generate_char_texture(&self, device: &Device, queue: &Queue, ch: char) -> GpuTexture {
        // Create a simple texture for the character
        // For now, we'll create a placeholder texture
        let width = self.char_width;
        let height = self.char_height;

        // Create a simple pattern for each character based on its ASCII value
        let mut pixels = vec![0u8; (width * height * 4) as usize]; // RGBA

        // Fill with a pattern based on the character
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;

                // Create a simple pattern - white pixels for visible chars, black for space
                let intensity = if ch != ' ' && (x % 2 == 0 || y % 2 == 0) {
                    255
                } else {
                    0
                };

                pixels[idx] = intensity; // R
                pixels[idx + 1] = intensity; // G
                pixels[idx + 2] = intensity; // B
                pixels[idx + 3] = 255; // A
            }
        }

        // Create a temporary TextureAsset to pass to GpuTexture::from_asset
        let temp_asset = TextureAsset {
            width,
            height,
            pixels,
        };

        GpuTexture::from_asset(device, queue, &temp_asset)
    }

    pub fn render_text(
        &self,
        text: &str,
        position: Vec2,
        color: [f32; 4],
        size: f32,
    ) -> Vec<RenderCommands> {
        let mut commands = Vec::new();

        let mut current_x = position.x;
        let y = position.y;

        for ch in text.chars() {
            if self.textures.contains_key(&ch) {
                // Calculate position for this character
                let char_pos = Vec2::new(current_x, y);

                // Add a text command that will be handled by the renderer
                // The renderer will need to know how to render each character
                commands.push(RenderCommands::Text {
                    text: ch.to_string(),
                    position: char_pos,
                    color,
                    size,
                });

                // Move to next character position
                current_x += (self.char_width as f32) * size * self.spacing;
            }
        }

        commands
    }
}

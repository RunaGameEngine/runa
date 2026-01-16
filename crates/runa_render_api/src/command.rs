use glam::Mat4;

use runa_asset::handle::Handle;
use runa_asset::texture::TextureAsset;

pub enum RenderCommands {
    Sprite {
        texture: Handle<TextureAsset>,
        model: Mat4,
    },
}

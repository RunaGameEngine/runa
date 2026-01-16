use crate::components::component::Component;
use runa_asset::handle::Handle;
use runa_asset::texture::TextureAsset;

pub struct SpriteRenderer {
    pub texture: Option<Handle<TextureAsset>>,
}

impl SpriteRenderer {
    pub fn new() -> Self {
        Self { texture: None }
    }
}

impl Component for SpriteRenderer {}

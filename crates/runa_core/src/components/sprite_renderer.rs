use crate::components::component::Component;
use runa_asset::Handle;
use runa_asset::TextureAsset;

pub struct SpriteRenderer {
    pub texture: Option<Handle<TextureAsset>>,
}

impl SpriteRenderer {
    /// texture = None
    pub fn default() -> Self {
        Self { texture: None }
    }

    pub fn get_texture_handle(&self) -> Handle<TextureAsset> {
        self.texture.clone().unwrap()
    }
}

impl Component for SpriteRenderer {}

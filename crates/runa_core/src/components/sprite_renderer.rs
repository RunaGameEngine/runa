use crate::components::component::Component;
use runa_asset::Handle;
use runa_asset::TextureAsset;

pub struct SpriteRenderer {
    pub texture: Option<Handle<TextureAsset>>,
    pub texture_path: Option<String>,
}

impl SpriteRenderer {
    pub fn new(texture: Option<Handle<TextureAsset>>) -> Self {
        let texture_path = texture
            .as_ref()
            .map(|handle| handle.inner.path.to_string_lossy().to_string());

        Self {
            texture,
            texture_path,
        }
    }

    /// texture = None
    pub fn default() -> Self {
        Self {
            texture: None,
            texture_path: None,
        }
    }

    pub fn get_texture_handle(&self) -> Handle<TextureAsset> {
        self.texture.clone().unwrap()
    }

    pub fn set_texture(
        &mut self,
        texture: Option<Handle<TextureAsset>>,
        texture_path: Option<String>,
    ) {
        self.texture = texture;
        self.texture_path = texture_path;
    }
}

impl Component for SpriteRenderer {}

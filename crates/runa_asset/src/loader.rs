use crate::handle::Handle;
use crate::texture::TextureAsset;

pub fn load_image(path: &str) -> Handle<TextureAsset> {
    let image = TextureAsset::load(path).expect("Failed to load image");
    Handle {
        inner: std::sync::Arc::new(image),
    }
}

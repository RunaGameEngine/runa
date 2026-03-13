use std::path::PathBuf;

use crate::handle::Handle;
use crate::texture::TextureAsset;

#[macro_export]
macro_rules! load_image {
    ($path:literal) => {{
        // Compile-time проверка
        const _: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path));

        // Runtime загрузка
        $crate::loader::load_image(env!("CARGO_MANIFEST_DIR"), $path)
    }};
}

pub fn load_image(cargo: &str, path: &str) -> Handle<TextureAsset> {
    let image = TextureAsset::load(&PathBuf::from(cargo).join(path)).expect("Failed to load image");
    Handle {
        inner: std::sync::Arc::new(image),
    }
}

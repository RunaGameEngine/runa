//! Runa Asset Loading — images, audio, and more
use std::path::PathBuf;

use crate::handle::Handle;
use crate::texture::TextureAsset;

pub use crate::audio::{AudioAsset, AudioLoadError};

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

/// Load audio asset at compile time (with caching)
#[macro_export]
macro_rules! load_audio {
    ($path:literal) => {{
        use std::sync::Arc;
        use std::sync::OnceLock;

        const _: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path));

        static CACHE: OnceLock<Arc<$crate::AudioAsset>> = OnceLock::new();

        CACHE
            .get_or_init(|| {
                Arc::new($crate::AudioAsset::from_file(env!("CARGO_MANIFEST_DIR"), $path).unwrap())
            })
            .clone()
    }};
}

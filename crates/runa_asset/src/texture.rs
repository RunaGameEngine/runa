use std::path::PathBuf;

pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA8
}

impl TextureAsset {
    pub fn load(path: &PathBuf) -> Result<Self, image::ImageError> {
        // println!("📂 Loading from: {:?}", path);
        let img = image::open(path)?.to_rgba8();

        let (width, height) = img.dimensions();

        Ok(Self {
            width,
            height,
            pixels: img.into_raw(),
        })
    }
}

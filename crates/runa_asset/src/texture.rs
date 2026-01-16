pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA8
}

impl TextureAsset {
    pub fn load(path: &str) -> Result<Self, image::ImageError> {
        let img = image::open(path)?.to_rgba8();
        let (width, height) = img.dimensions();

        Ok(Self {
            width,
            height,
            pixels: img.into_raw(),
        })
    }
}

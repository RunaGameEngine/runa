use std::sync::Arc;

pub fn load_icon(ctx: &egui::Context, bytes: &[u8]) -> Option<Arc<egui::TextureHandle>> {
    match image::load_from_memory(bytes) {
        Ok(image) => {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let texture = ctx.load_texture(
                "icon",
                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                Default::default(),
            );
            Some(Arc::new(texture))
        }
        Err(e) => {
            eprintln!("Failed to load icon: {}", e);
            None
        }
    }
}

pub fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes).ok()?;
    let image = image.to_rgba8();
    let (width, height) = image.dimensions();

    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

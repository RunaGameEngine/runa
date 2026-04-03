#[cfg(windows)]
fn main() {
    use std::path::PathBuf;

    let icon_path = PathBuf::from("assets").join("icon.png");
    println!("cargo:rerun-if-changed={}", icon_path.display());

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR must exist"));
    let ico_path = out_dir.join("runa_editor.ico");

    let image = image::open(&icon_path)
        .expect("failed to load editor icon")
        .resize_exact(256, 256, image::imageops::FilterType::Lanczos3);

    image
        .save_with_format(&ico_path, image::ImageFormat::Ico)
        .expect("failed to convert icon.png to .ico");

    winres::WindowsResource::new()
        .set_icon(ico_path.to_string_lossy().as_ref())
        .compile()
        .expect("failed to embed windows icon resource");
}

#[cfg(not(windows))]
fn main() {}

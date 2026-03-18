use std::path::Path;
use winit::window::Icon;

/// Загружает иконку окна из файла изображения
///
/// # Требования
/// - Формат: PNG (рекомендуется)
/// - Размер: кратен 16 (16x16, 32x32, 64x64, 128x128, 256x256)
/// - Каналы: RGBA (с альфа-каналом)
///
/// # Пример
/// ```rust
/// let icon = load_window_icon("assets/icon.png")?;
/// window.set_window_icon(Some(icon));
/// ```
pub fn load_window_icon<P: AsRef<Path>>(path: P) -> Result<Icon, String> {
    let path = path.as_ref();

    // Загружаем изображение через image crate
    let img = image::open(path)
        .map_err(|e| format!("Failed to load icon '{}': {}", path.display(), e))?;

    // Конвертируем в RGBA8
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Проверяем размер (winit требует квадратные иконки)
    if width != height {
        return Err(format!("Icon must be square, got {}x{}", width, height));
    }

    // Проверяем поддерживаемые размеры
    const VALID_SIZES: &[u32] = &[16, 32, 48, 64, 128, 256, 512];
    if !VALID_SIZES.contains(&width) {
        return Err(format!(
            "Icon size {}x{} is not standard. Recommended: 16, 32, 64, 128, 256, 512",
            width, height
        ));
    }

    // Создаём Icon для winit
    Icon::from_rgba(rgba.to_vec(), width, height)
        .map_err(|e| format!("Failed to create icon: {}", e))
}

/// Загружает несколько иконок разных размеров (рекомендуется для кроссплатформенности)
///
/// # Пример
/// ```rust
/// let icons = load_window_icons(&[
///     "assets/icon_16.png",
///     "assets/icon_32.png",
///     "assets/icon_64.png",
///     "assets/icon_256.png",
/// ])?;
/// window.set_window_icon(icons.first().cloned()); // winit использует первую подходящую
/// ```
pub fn load_window_icons<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<Icon>, String> {
    let mut icons = Vec::new();

    for path in paths {
        match load_window_icon(path) {
            Ok(icon) => icons.push(icon),
            Err(e) => eprintln!("⚠️  Skipping icon '{}': {}", path.as_ref().display(), e),
        }
    }

    if icons.is_empty() {
        return Err("No valid icons loaded".to_string());
    }

    Ok(icons)
}

use std::sync::Arc;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::resources::texture::GpuTexture;
use runa_asset::TextureAsset;
use rusttype::{point, Font, Scale};
use wgpu::{Device, Queue};

/// UV coordinates for a character in the atlas
#[derive(Clone, Copy, Debug)]
pub struct CharUV {
    pub u: f32,
    pub v: f32,
    pub u_width: f32,
    pub v_height: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub uv: CharUV,
    pub advance: f32,
}

pub struct FontManager {
    atlas_texture: Option<Arc<GpuTexture>>,
    glyphs: HashMap<char, GlyphInfo>,
    cell_width: f32,
    cell_height: f32,
    line_height: f32,
    ascent: f32,
    descent: f32,
    base_font_size: f32,
    atlas_width: u32,
    atlas_height: u32,
}

impl FontManager {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        if let Some(font_bytes) = Self::load_default_system_font_bytes() {
            return Self::new_ttf(device, queue, font_bytes, 32.0);
        }

        Self::new_bitmap(device, queue)
    }

    pub fn new_ttf(device: &Device, queue: &Queue, font_bytes: Vec<u8>, font_size: f32) -> Self {
        let font = Font::try_from_vec(font_bytes).expect("Failed to parse TTF font data");
        let scale = Scale::uniform(font_size);
        let v_metrics = font.v_metrics(scale);
        let ascent = v_metrics.ascent;
        let descent = v_metrics.descent.abs();
        let line_height = (ascent + descent).ceil() as u32;
        let chars: Vec<char> = (32u8..127u8).map(|c| c as char).collect();

        let mut max_advance: f32 = 0.0;
        for ch in chars.iter() {
            let glyph = font.glyph(*ch).scaled(scale);
            max_advance = max_advance.max(glyph.h_metrics().advance_width);
        }

        let padding = 2usize;
        let cell_width = max_advance.ceil() as usize + padding * 2;
        let cell_height = line_height as usize + padding * 2;
        let cols = 16usize;
        let rows = chars.len().div_ceil(cols);
        let atlas_width = cols * cell_width;
        let atlas_height = rows * cell_height;

        let pixel_count = atlas_width
            .checked_mul(atlas_height)
            .and_then(|count| count.checked_mul(4))
            .expect("Font atlas size overflowed");
        let mut pixels = vec![0u8; pixel_count];
        let mut glyphs = HashMap::new();

        for (index, ch) in chars.iter().enumerate() {
            let col = index % cols;
            let row = index / cols;
            let cell_x = col * cell_width;
            let cell_y = row * cell_height;

            let glyph = font.glyph(*ch).scaled(scale).positioned(point(0.0, ascent));
            let advance = glyph.unpositioned().h_metrics().advance_width;

            let uv = if let Some(bb) = glyph.pixel_bounding_box() {
                let atlas_x = cell_x as i32 + padding as i32;
                let atlas_y = cell_y as i32 + padding as i32;

                glyph.draw(|x, y, v| {
                    let px_i32 = atlas_x + x as i32;
                    let py_i32 = atlas_y + y as i32;
                    if px_i32 >= 0 && py_i32 >= 0 {
                        let px = px_i32 as usize;
                        let py = py_i32 as usize;
                        if px < atlas_width && py < atlas_height {
                            let idx = (py * atlas_width + px) * 4;
                            let intensity = (v * 255.0).round() as u8;
                            pixels[idx] = intensity;
                            pixels[idx + 1] = intensity;
                            pixels[idx + 2] = intensity;
                            pixels[idx + 3] = intensity;
                        }
                    }
                });

                CharUV {
                    u: atlas_x as f32 / atlas_width as f32,
                    v: atlas_y as f32 / atlas_height as f32,
                    u_width: bb.width() as f32 / atlas_width as f32,
                    v_height: bb.height() as f32 / atlas_height as f32,
                    bearing_x: padding as f32 + bb.min.x as f32,
                    bearing_y: padding as f32 + bb.min.y as f32,
                    width: bb.width() as f32,
                    height: bb.height() as f32,
                }
            } else {
                CharUV {
                    u: cell_x as f32 / atlas_width as f32,
                    v: cell_y as f32 / atlas_height as f32,
                    u_width: 0.0,
                    v_height: 0.0,
                    bearing_x: 0.0,
                    bearing_y: 0.0,
                    width: 0.0,
                    height: 0.0,
                }
            };

            glyphs.insert(
                *ch,
                GlyphInfo {
                    uv,
                    advance: if advance > 0.0 {
                        advance
                    } else {
                        cell_width as f32 * 0.5
                    },
                },
            );
        }

        let temp_asset = TextureAsset {
            width: atlas_width
                .try_into()
                .expect("Font atlas width exceeds u32"),
            height: atlas_height
                .try_into()
                .expect("Font atlas height exceeds u32"),
            pixels,
            path: PathBuf::new(),
        };

        let atlas_texture = Arc::new(GpuTexture::from_asset(device, queue, &temp_asset));
        Self {
            atlas_texture: Some(atlas_texture),
            glyphs,
            cell_width: cell_width as f32,
            cell_height: cell_height as f32,
            line_height: line_height as f32,
            ascent,
            descent,
            base_font_size: font_size,
            atlas_width: atlas_width
                .try_into()
                .expect("Font atlas width exceeds u32"),
            atlas_height: atlas_height
                .try_into()
                .expect("Font atlas height exceeds u32"),
        }
    }

    fn new_bitmap(device: &Device, queue: &Queue) -> Self {
        let char_width = 8u32;
        let char_height = 8u32;
        let cols = 16u32;
        let rows = 6u32;
        let atlas_width = cols * char_width;
        let atlas_height = rows * char_height;

        let mut font_manager = Self {
            atlas_texture: None,
            glyphs: HashMap::new(),
            cell_width: char_width as f32,
            cell_height: char_height as f32,
            line_height: char_height as f32,
            ascent: char_height as f32,
            descent: 0.0,
            base_font_size: char_height as f32,
            atlas_width,
            atlas_height,
        };

        let texture = font_manager.generate_bitmap_atlas(device, queue);
        font_manager.atlas_texture = Some(Arc::new(texture));
        font_manager
    }

    fn load_default_system_font_bytes() -> Option<Vec<u8>> {
        const WINDOWS_FONTS: &[&str] = &[
            "C:\\Windows\\Fonts\\arial.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\calibri.ttf",
        ];
        const MACOS_FONTS: &[&str] = &["/Library/Fonts/Arial.ttf", "/Library/Fonts/Helvetica.ttf"];
        const LINUX_FONTS: &[&str] = &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ];

        let font_paths = if cfg!(target_os = "windows") {
            WINDOWS_FONTS
        } else if cfg!(target_os = "macos") {
            MACOS_FONTS
        } else if cfg!(target_os = "linux") {
            LINUX_FONTS
        } else {
            &[]
        };

        for path in font_paths {
            if let Ok(bytes) = fs::read(path) {
                return Some(bytes);
            }
        }

        None
    }

    fn generate_bitmap_atlas(&mut self, device: &Device, queue: &Queue) -> GpuTexture {
        let cols = self.atlas_width / self.cell_width as u32;
        let mut pixels = vec![0u8; (self.atlas_width * self.atlas_height * 4) as usize];
        let char_bitmaps: &[(u8, [u8; 8])] = &[
            (b' ', [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (b'!', [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00]),
            (b'"', [0x36, 0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (b'#', [0x00, 0x36, 0x36, 0x7F, 0x36, 0x7F, 0x36, 0x36]),
            (b'$', [0x08, 0x7F, 0x49, 0x2A, 0x12, 0x7F, 0x10, 0x00]),
            (b'%', [0x00, 0x46, 0x4A, 0x08, 0x10, 0x28, 0x64, 0x00]),
            (b'&', [0x30, 0x48, 0x30, 0x2A, 0x45, 0x42, 0x3D, 0x00]),
            (
                b'\'',
                [0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00],
            ),
            (b'(', [0x0C, 0x18, 0x30, 0x30, 0x30, 0x30, 0x18, 0x0C]),
            (b')', [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x0C, 0x18, 0x30]),
            (b'*', [0x00, 0x08, 0x2A, 0x1C, 0x1C, 0x2A, 0x08, 0x00]),
            (b'+', [0x00, 0x08, 0x08, 0x3E, 0x08, 0x08, 0x00, 0x00]),
            (b',', [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30]),
            (b'-', [0x00, 0x00, 0x00, 0x3E, 0x00, 0x00, 0x00, 0x00]),
            (b'.', [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00]),
            (b'/', [0x00, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x00]),
            (b'0', [0x3C, 0x46, 0x4A, 0x52, 0x62, 0x46, 0x3C, 0x00]),
            (b'1', [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00]),
            (b'2', [0x3C, 0x46, 0x02, 0x0C, 0x30, 0x40, 0x7E, 0x00]),
            (b'3', [0x3C, 0x46, 0x02, 0x1C, 0x02, 0x46, 0x3C, 0x00]),
            (b'4', [0x06, 0x0E, 0x1E, 0x26, 0x7F, 0x06, 0x06, 0x00]),
            (b'5', [0x7E, 0x40, 0x7C, 0x02, 0x02, 0x46, 0x3C, 0x00]),
            (b'6', [0x3C, 0x40, 0x7C, 0x46, 0x46, 0x46, 0x3C, 0x00]),
            (b'7', [0x7E, 0x02, 0x04, 0x08, 0x10, 0x10, 0x10, 0x00]),
            (b'8', [0x3C, 0x46, 0x46, 0x3C, 0x46, 0x46, 0x3C, 0x00]),
            (b'9', [0x3C, 0x46, 0x46, 0x3E, 0x02, 0x04, 0x38, 0x00]),
            (b':', [0x00, 0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00]),
            (b';', [0x00, 0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x30]),
            (b'<', [0x04, 0x08, 0x10, 0x20, 0x10, 0x08, 0x04, 0x00]),
            (b'=', [0x00, 0x00, 0x7E, 0x00, 0x00, 0x7E, 0x00, 0x00]),
            (b'>', [0x20, 0x10, 0x08, 0x04, 0x08, 0x10, 0x20, 0x00]),
            (b'?', [0x3C, 0x46, 0x02, 0x0C, 0x18, 0x00, 0x18, 0x00]),
            (b'@', [0x3C, 0x46, 0x5C, 0x5A, 0x5A, 0x5C, 0x3C, 0x00]),
            (b'A', [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00]),
            (b'B', [0x7C, 0x46, 0x46, 0x7C, 0x46, 0x46, 0x7C, 0x00]),
            (b'C', [0x3C, 0x46, 0x40, 0x40, 0x40, 0x46, 0x3C, 0x00]),
            (b'D', [0x78, 0x4C, 0x46, 0x46, 0x46, 0x4C, 0x78, 0x00]),
            (b'E', [0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x7E, 0x00]),
            (b'F', [0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x40, 0x00]),
            (b'G', [0x3C, 0x46, 0x40, 0x4E, 0x46, 0x46, 0x3C, 0x00]),
            (b'H', [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00]),
            (b'I', [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00]),
            (b'J', [0x02, 0x02, 0x02, 0x02, 0x02, 0x46, 0x3C, 0x00]),
            (b'K', [0x66, 0x4C, 0x38, 0x38, 0x38, 0x4C, 0x66, 0x00]),
            (b'L', [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00]),
            (b'M', [0x63, 0x77, 0x7F, 0x7F, 0x6B, 0x63, 0x63, 0x00]),
            (b'N', [0x66, 0x66, 0x76, 0x7E, 0x6E, 0x66, 0x66, 0x00]),
            (b'O', [0x3C, 0x46, 0x46, 0x46, 0x46, 0x46, 0x3C, 0x00]),
            (b'P', [0x7C, 0x46, 0x46, 0x7C, 0x40, 0x40, 0x40, 0x00]),
            (b'Q', [0x3C, 0x46, 0x46, 0x46, 0x4A, 0x4C, 0x36, 0x00]),
            (b'R', [0x7C, 0x46, 0x46, 0x7C, 0x38, 0x4C, 0x66, 0x00]),
            (b'S', [0x3C, 0x46, 0x40, 0x3C, 0x06, 0x46, 0x3C, 0x00]),
            (b'T', [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00]),
            (b'U', [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00]),
            (b'V', [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00]),
            (b'W', [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00]),
            (b'X', [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00]),
            (b'Y', [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00]),
            (b'Z', [0x7E, 0x04, 0x08, 0x10, 0x20, 0x40, 0x7E, 0x00]),
            (b'[', [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00]),
            (
                b'\\',
                [0x00, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x00],
            ),
            (b']', [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00]),
            (b'^', [0x18, 0x3C, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (b'_', [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E]),
            (b'`', [0x18, 0x1C, 0x0E, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (b'a', [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x46, 0x3E, 0x00]),
            (b'b', [0x40, 0x40, 0x7C, 0x46, 0x46, 0x46, 0x7C, 0x00]),
            (b'c', [0x00, 0x00, 0x3C, 0x40, 0x40, 0x46, 0x3C, 0x00]),
            (b'd', [0x02, 0x02, 0x3E, 0x46, 0x46, 0x46, 0x3E, 0x00]),
            (b'e', [0x00, 0x00, 0x3C, 0x46, 0x7E, 0x40, 0x3C, 0x00]),
            (b'f', [0x0C, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x00]),
            (b'g', [0x00, 0x00, 0x3E, 0x46, 0x46, 0x3E, 0x06, 0x3C]),
            (b'h', [0x40, 0x40, 0x7C, 0x46, 0x46, 0x46, 0x46, 0x00]),
            (b'i', [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00]),
            (b'j', [0x06, 0x00, 0x0E, 0x06, 0x06, 0x06, 0x06, 0x3C]),
            (b'k', [0x40, 0x40, 0x46, 0x4C, 0x78, 0x4C, 0x46, 0x00]),
            (b'l', [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00]),
            (b'm', [0x00, 0x00, 0x5A, 0x7F, 0x6B, 0x63, 0x63, 0x00]),
            (b'n', [0x00, 0x00, 0x7C, 0x46, 0x46, 0x46, 0x46, 0x00]),
            (b'o', [0x00, 0x00, 0x3C, 0x46, 0x46, 0x46, 0x3C, 0x00]),
            (b'p', [0x00, 0x00, 0x7C, 0x46, 0x46, 0x7C, 0x40, 0x40]),
            (b'q', [0x00, 0x00, 0x3E, 0x46, 0x46, 0x3E, 0x06, 0x06]),
            (b'r', [0x00, 0x00, 0x6E, 0x38, 0x18, 0x18, 0x18, 0x00]),
            (b's', [0x00, 0x00, 0x3E, 0x40, 0x3C, 0x06, 0x7C, 0x00]),
            (b't', [0x18, 0x18, 0x7E, 0x18, 0x18, 0x1A, 0x0C, 0x00]),
            (b'u', [0x00, 0x00, 0x46, 0x46, 0x46, 0x46, 0x3E, 0x00]),
            (b'v', [0x00, 0x00, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00]),
            (b'w', [0x00, 0x00, 0x63, 0x63, 0x6B, 0x7F, 0x3E, 0x00]),
            (b'x', [0x00, 0x00, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x00]),
            (b'y', [0x00, 0x00, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x3C]),
            (b'z', [0x00, 0x00, 0x7E, 0x0C, 0x18, 0x30, 0x7E, 0x00]),
            (b'{', [0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0x00]),
            (b'|', [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18]),
            (b'}', [0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0x00]),
            (b'~', [0x00, 0x00, 0x00, 0x6E, 0x31, 0x00, 0x00, 0x00]),
        ];

        for c in 32u8..127 {
            let ch = c as char;
            let char_idx = (c - 32) as usize;
            let col = (char_idx % cols as usize) as u32;
            let row = (char_idx / cols as usize) as u32;
            let u = (col * self.cell_width as u32) as f32 / self.atlas_width as f32;
            let v = (row * self.cell_height as u32) as f32 / self.atlas_height as f32;
            let u_width = self.cell_width / self.atlas_width as f32;
            let v_height = self.cell_height / self.atlas_height as f32;
            self.glyphs.insert(
                ch,
                GlyphInfo {
                    uv: CharUV {
                        u,
                        v,
                        u_width,
                        v_height,
                        bearing_x: 0.0,
                        bearing_y: 0.0,
                        width: self.cell_width,
                        height: self.cell_height,
                    },
                    advance: self.cell_width,
                },
            );

            let bitmap = char_bitmaps
                .iter()
                .find(|(code, _)| *code == c)
                .map(|(_, bmp)| *bmp)
                .unwrap_or([0x00; 8]);

            for y in 0..self.cell_height as u32 {
                for x in 0..self.cell_width as u32 {
                    let atlas_x = col * self.cell_width as u32 + x;
                    let atlas_y = row * self.cell_height as u32 + y;
                    let idx = ((atlas_y * self.atlas_width + atlas_x) * 4) as usize;
                    let bit = 7 - (x % 8);
                    let pixel_on = (bitmap[y as usize] >> bit) & 1 != 0;
                    let intensity = if pixel_on { 255 } else { 0 };
                    pixels[idx] = intensity;
                    pixels[idx + 1] = intensity;
                    pixels[idx + 2] = intensity;
                    pixels[idx + 3] = 255;
                }
            }
        }

        let temp_asset = TextureAsset {
            width: self.atlas_width,
            height: self.atlas_height,
            pixels,
            path: PathBuf::new(),
        };

        GpuTexture::from_asset(device, queue, &temp_asset)
    }

    pub fn get_atlas_texture(&self) -> Option<&Arc<GpuTexture>> {
        self.atlas_texture.as_ref()
    }

    pub fn get_char_uv(&self, ch: char) -> Option<CharUV> {
        self.glyphs.get(&ch).map(|info| info.uv)
    }

    pub fn get_glyph_info(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyphs.get(&ch)
    }

    pub fn get_char_advance(&self, ch: char) -> Option<f32> {
        self.glyphs.get(&ch).map(|info| info.advance)
    }

    pub fn char_size(&self) -> (u32, u32) {
        (self.cell_width as u32, self.cell_height as u32)
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn ascent(&self) -> f32 {
        self.ascent
    }

    pub fn descent(&self) -> f32 {
        self.descent
    }

    pub fn base_font_size(&self) -> f32 {
        self.base_font_size
    }
}

use glam::{IVec2, USizeVec2, Vec3};
use runa_asset::TextureAsset;
use std::sync::Arc;

/// Прямоугольник для UV-координат и позиционирования
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Один тайл в тайлмапе
#[derive(Clone)]
pub struct Tile {
    pub texture: Option<Arc<TextureAsset>>, // None = пустой тайл
    pub uv_rect: Rect,                      // часть текстуры (атласа)
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Tile {
    pub fn new(texture: Arc<TextureAsset>, uv_rect: Rect) -> Self {
        Self {
            texture: Some(texture),
            uv_rect,
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn empty() -> Self {
        Self {
            texture: None,
            uv_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            flip_x: false,
            flip_y: false,
        }
    }
}

/// Один слой тайлмапа
#[derive(Clone)]
pub struct TilemapLayer {
    pub name: String,
    pub tiles: Vec<Tile>, // width * height элементов
    pub visible: bool,
    pub opacity: f32,
}

impl TilemapLayer {
    pub fn new(name: String, width: u32, height: u32) -> Self {
        Self {
            name,
            tiles: vec![Tile::empty(); (width * height) as usize],
            visible: true,
            opacity: 1.0,
        }
    }

    pub fn set_tile(&mut self, x: u32, y: u32, width: u32, tile: Tile) {
        let index = (y * width + x) as usize;
        if index < self.tiles.len() {
            self.tiles[index] = tile;
        }
    }

    pub fn get_tile(&self, x: u32, y: u32, width: u32) -> Option<&Tile> {
        let index = (y * width + x) as usize;
        self.tiles.get(index)
    }
}

/// Компонент тайлмапа (данные)
#[derive(Clone)]
pub struct Tilemap {
    /// Размер карты в тайлах
    pub width: u32,
    pub height: u32,

    /// Размер одного тайла в мировых единицах
    /// Например: 16 пикселей при pixels_per_unit=16 → tile_size=1.0
    pub tile_size: USizeVec2,
    pub offset: IVec2,

    /// Слои (от дальнего к ближнему)
    pub layers: Vec<TilemapLayer>,
}

impl Tilemap {
    /// Создать карту с центром в (0, 0) мира
    pub fn centered(width: u32, height: u32, tile_size: USizeVec2) -> Self {
        let offset = IVec2::new(-(width as i32) / 2, -(height as i32) / 2);

        Self {
            width,
            height,
            tile_size,
            offset,
            layers: Vec::new(),
        }
    }

    pub fn add_layer(&mut self, layer: TilemapLayer) {
        self.layers.push(layer);
    }

    /// Установить тайл по мировым координатам (в тайлах, могут быть отрицательными)
    pub fn set_tile(&mut self, world_x: i32, world_y: i32, tile: Tile) {
        // Конвертируем мировые координаты → индексы массива
        let array_x = (world_x - self.offset.x) as u32;
        let array_y = (world_y - self.offset.y) as u32;

        // Проверка границ
        if array_x < self.width && array_y < self.height {
            for layer in &mut self.layers {
                layer.set_tile(array_x, array_y, self.width, tile.clone());
            }
        }
    }

    /// Получить тайл по мировым координатам
    pub fn get_tile(&self, world_x: i32, world_y: i32) -> Option<&Tile> {
        let array_x = (world_x - self.offset.x) as u32;
        let array_y = (world_y - self.offset.y) as u32;

        if array_x < self.width && array_y < self.height {
            self.layers.first()?.get_tile(array_x, array_y, self.width)
        } else {
            None
        }
    }

    /// Конвертировать мировые координаты → координаты тайла
    pub fn world_to_tile(&self, world_pos: Vec3) -> (i32, i32) {
        let tile_x = (world_pos.x / self.tile_size.x as f32).floor() as i32;
        let tile_y = (world_pos.y / self.tile_size.y as f32).floor() as i32;
        (tile_x, tile_y)
    }

    /// Конвертировать координаты тайла → мировые координаты центра тайла
    pub fn tile_to_world(&self, tile_x: i32, tile_y: i32) -> Vec3 {
        Vec3::new(
            (tile_x as f32 + 0.5) * self.tile_size.x as f32,
            (tile_y as f32 + 0.5) * self.tile_size.y as f32,
            0.0,
        )
    }
}

/// Компонент рендеринга тайлмапа
#[derive(Clone)]
pub struct TilemapRenderer;

impl TilemapRenderer {
    pub fn new() -> Self {
        Self
    }
}

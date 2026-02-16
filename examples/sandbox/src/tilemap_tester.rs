use std::sync::Arc;

use glam::USizeVec2;
use runa_core::{
    components::{Rect, Tile, Tilemap, TilemapLayer, TilemapRenderer, Transform},
    ocs::Script,
};

pub struct TilemapTester {}

impl TilemapTester {
    pub fn new() -> Self {
        Self {}
    }
}

impl Script for TilemapTester {
    fn construct(&self, _object: &mut runa_core::ocs::Object) {
        _object.add_component(Transform::default());
        _object.add_component({
            let mut tilemap = Tilemap::centered(25, 25, USizeVec2::new(2, 2));
            let mut layer = TilemapLayer::new("Test".to_string(), 25, 25);

            let grass_texture = runa_asset::loader::load_image("assets/TilemapTest.png");
            let dirt_texture = runa_asset::loader::load_image("assets/TilemapTest2.png");

            for y in 0..25 {
                for x in 0..25 {
                    let texture = if (x + y) % 2 == 0 {
                        grass_texture.clone()
                    } else {
                        dirt_texture.clone()
                    };

                    let tile = Tile::new(Arc::from(texture), Rect::new(0.0, 0.0, 1.0, 1.0));
                    layer.set_tile(x, y, 10, tile);
                }
            }
            tilemap.add_layer(layer);
            tilemap
        });

        _object.add_component(TilemapRenderer::new());
    }

    fn start(&mut self, _object: &mut runa_core::ocs::Object) {}

    fn update(&mut self, _object: &mut runa_core::ocs::Object, _dt: f32) {}
}

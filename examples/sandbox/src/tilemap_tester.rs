use std::sync::Arc;

use runa_core::glam::USizeVec2;
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
            let tilesize = 10;
            let mut tilemap = Tilemap::centered(tilesize, tilesize, USizeVec2::new(2, 2));
            let mut layer = TilemapLayer::new("Test".into(), tilesize, tilesize);
            let mut layer2 = TilemapLayer::new("Test2".into(), tilesize, tilesize);

            let grass_texture = runa_asset::loader::load_image("assets/TilemapTest.png");
            let trans_tile_texture = runa_asset::loader::load_image("assets/TilemapTestTrans.png");

            for y in 0..tilesize {
                for x in 0..tilesize {
                    let tile = Tile::new(
                        Arc::from(grass_texture.clone()),
                        Rect::new(0.0, 0.0, 1.0, 1.0),
                    );
                    let tile2 = Tile::new(
                        Arc::from(trans_tile_texture.clone()),
                        Rect::new(0.0, 0.0, 1.0, 1.0),
                    );
                    layer.set_tile(x, y, tile);
                    layer2.set_tile(x, y, tile2);
                }
            }
            tilemap.add_layer(layer);
            tilemap.add_layer(layer2);
            tilemap
        });

        _object.add_component(TilemapRenderer::new());
    }

    fn start(&mut self, _object: &mut runa_core::ocs::Object) {}

    fn update(&mut self, _object: &mut runa_core::ocs::Object, _dt: f32) {}
}

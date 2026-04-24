use std::sync::Arc;

use runa_core::glam::USizeVec2;
use runa_core::{
    components::{Rect, Tile, Tilemap, TilemapLayer, TilemapRenderer},
    ocs::{Object, World},
};
use runa_engine::RunaArchetype;

pub fn create_tilemap_tester() -> Object {
    let tilemap = {
        let tilesize = 10;
        let mut tilemap = Tilemap::centered(tilesize, tilesize, USizeVec2::new(32, 32));
        let mut layer = TilemapLayer::new("Test".into(), tilesize, tilesize);
        let mut layer2 = TilemapLayer::new("Test2".into(), tilesize, tilesize);

        let grass_texture = runa_asset::load_image!("assets/art/TilemapTest.png");
        let trans_tile_texture = runa_asset::load_image!("assets/art/TilemapTestTrans.png");

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
    };

    Object::new("Tilemap")
        .with(tilemap)
        .with(TilemapRenderer::new())
}

#[derive(RunaArchetype)]
#[runa(name = "tilemap_tester")]
pub struct TilemapTesterArchetype;

impl TilemapTesterArchetype {
    pub fn create(world: &mut World) -> u64 {
        world.spawn(create_tilemap_tester())
    }
}

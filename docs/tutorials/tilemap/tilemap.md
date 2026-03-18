# Tilemap System

Tilemaps let you create 2D levels using a grid of tiles. This tutorial shows you how to create and use tilemaps.

## Quick Start

### Step 1: Create a Tilemap

```rust
use runa_core::components::{Tilemap, TilemapLayer, Tile, Rect};
use runa_core::glam::USizeVec2;
use std::sync::Arc;

// Create a 10x10 tilemap with 16x16 pixel tiles
let mut tilemap = Tilemap::centered(10, 10, USizeVec2::new(16, 16));

// Create a layer
let mut layer = TilemapLayer::new("Ground".to_string(), 10, 10);

// Load a tile texture
let tile_texture = runa_asset::load_image!("assets/tiles/grass.png");

// Set tiles
for y in 0..10 {
    for x in 0..10 {
        let tile = Tile::new(
            Arc::from(tile_texture.clone()),
            Rect::new(0.0, 0.0, 1.0, 1.0),
        );
        layer.set_tile(x, y, tile);
    }
}

// Add layer to tilemap
tilemap.add_layer(layer);

// Add to object
object.add_component(tilemap);
object.add_component(TilemapRenderer::new());
```

## Tilemap Structure

A tilemap consists of:

- **Tilemap** - The overall grid and settings
- **Layers** - Multiple stacked grids (ground, decorations, etc.)
- **Tiles** - Individual cells with textures

## Creating Layers

```rust
// Create multiple layers
let mut ground_layer = TilemapLayer::new("Ground".to_string(), 20, 20);
let mut decoration_layer = TilemapLayer::new("Decorations".to_string(), 20, 20);

// Layers are rendered in order (first = bottom)
tilemap.add_layer(ground_layer);
tilemap.add_layer(decoration_layer);
```

## Working with Tiles

### Setting Tiles

```rust
// Set a single tile
let grass_tile = Tile::new(
    Arc::from(grass_texture),
    Rect::new(0.0, 0.0, 1.0, 1.0),
);
layer.set_tile(5, 5, grass_tile);

// Set tile with UV coordinates (for texture atlases)
let water_tile = Tile::new(
    Arc::from(water_texture),
    Rect::new(0.5, 0.0, 0.5, 0.5), // Use half of texture
);
layer.set_tile(3, 3, water_tile);
```

### Getting Tiles

```rust
if let Some(tile) = layer.get_tile(5, 5) {
    println!("Tile at (5, 5): {:?}", tile);
}
```

### Empty Tiles

```rust
// Create an empty (transparent) tile
let empty = Tile::empty();
layer.set_tile(0, 0, empty);
```

## Tile Coordinates

### World to Tile

```rust
// Convert world position to tile coordinates
let world_pos = Vec3::new(32.0, 48.0, 0.0);
let (tile_x, tile_y) = tilemap.world_to_tile(world_pos);
```

### Tile to World

```rust
// Get world position of tile center
let world_pos = tilemap.tile_to_world(5, 3);
```

## Complete Example: Simple Level

```rust
use runa_core::{
    components::{Tilemap, TilemapLayer, TilemapRenderer, Tile, Rect},
    ocs::{Object, Script},
    World,
    glam::USizeVec2,
};
use std::sync::Arc;

pub struct LevelLoader;

impl LevelLoader {
    pub fn new() -> Self {
        Self
    }
}

impl Script for LevelLoader {
    fn construct(&self, object: &mut Object) {
        object.add_component(Transform::default());
        
        // Create tilemap
        let mut tilemap = Tilemap::centered(10, 10, USizeVec2::new(32, 32));
        
        // Load textures
        let grass_tex = runa_asset::load_image!("assets/tiles/grass.png");
        let water_tex = runa_asset::load_image!("assets/tiles/water.png");
        
        // Create layer
        let mut layer = TilemapLayer::new("Ground".to_string(), 10, 10);
        
        // Fill with grass
        for y in 0..10 {
            for x in 0..10 {
                let tile = Tile::new(
                    Arc::from(grass_tex.clone()),
                    Rect::new(0.0, 0.0, 1.0, 1.0),
                );
                layer.set_tile(x, y, tile);
            }
        }
        
        // Add water in center
        for y in 3..7 {
            for x in 3..7 {
                let tile = Tile::new(
                    Arc::from(water_tex.clone()),
                    Rect::new(0.0, 0.0, 1.0, 1.0),
                );
                layer.set_tile(x, y, tile);
            }
        }
        
        tilemap.add_layer(layer);
        object.add_component(tilemap);
        object.add_component(TilemapRenderer::new());
    }
}
```

## Properties

### Tilemap

| Property | Type | Description |
|----------|------|-------------|
| `width` | `u32` | Width in tiles |
| `height` | `u32` | Height in tiles |
| `tile_size` | `USizeVec2` | Size of each tile in pixels |
| `layers` | `Vec<TilemapLayer>` | List of layers |

### TilemapLayer

| Property | Type | Description |
|----------|------|-------------|
| `name` | `String` | Layer name |
| `width` | `u32` | Layer width |
| `height` | `u32` | Layer height |
| `visible` | `bool` | Is layer visible |
| `opacity` | `f32` | Layer transparency (0.0-1.0) |

### Tile

| Property | Type | Description |
|----------|------|-------------|
| `texture` | `Option<Arc<TextureAsset>>` | Tile image |
| `uv_rect` | `Rect` | UV coordinates for texture atlas |
| `flip_x` | `bool` | Flip horizontally |
| `flip_y` | `bool` | Flip vertically |

## Tips

- Use layers to separate ground, decorations, and collisions
- Texture atlases improve performance (multiple tiles in one image)
- Keep tile sizes as powers of 2 (16, 32, 64)
- Use `TilemapRenderer` component to render the tilemap

## Next Steps

- [SpriteRenderer](../components/sprite-renderer.md) for character sprites
- [Transform](../components/transform.md) for positioning
- [PhysicsCollision](../components/physics-collision.md) for tile collisions

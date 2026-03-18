# SpriteRenderer Component

The `SpriteRenderer` component displays an image (texture) on the screen.

## Adding a SpriteRenderer

```rust
use runa_core::components::SpriteRenderer;

object.add_component(SpriteRenderer {
    texture: Some(runa_asset::load_image!("assets/sprite.png")),
});
```

## Loading Images

Use the `load_image!` macro to load images at compile time:

```rust
// Load an image from your assets folder
let texture = runa_asset::load_image!("assets/character.png");

// Add to SpriteRenderer
object.add_component(SpriteRenderer {
    texture: Some(texture),
});
```

## Creating Without a Texture

```rust
object.add_component(SpriteRenderer {
    texture: None, // No texture (invisible)
});
```

## Complete Example: Player Sprite

```rust
use runa_core::{
    components::{SpriteRenderer, Transform},
    ocs::{Object, Script},
    World,
    glam::Vec3,
};

pub struct Player {
    speed: f32,
}

impl Player {
    pub fn new() -> Self {
        Self { speed: 3.0 }
    }
}

impl Script for Player {
    fn construct(&self, object: &mut Object) {
        object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::load_image!("assets/player.png")),
            });
    }

    fn start(&mut self, object: &mut Object) {
        if let Some(transform) = object.get_component_mut::<Transform>() {
            transform.position = Vec3::new(0.0, 0.0, 0.0);
        }
    }

    fn update(&mut self, object: &mut Object, dt: f32, _world: &mut World) {
        // Player logic here
    }
}
```

## File Paths

Image paths are relative to your project's `Cargo.toml`:

```rust
// For a project with this structure:
// my_game/
//   Cargo.toml
//   assets/
//     sprites/
//       player.png

runa_asset::load_image!("assets/sprites/player.png")
```

## Supported Formats

The engine supports common image formats:
- PNG (recommended)
- JPG
- GIF
- BMP
- TGA

## Tips

- PNG is recommended for game assets (supports transparency)
- Keep sprite sizes as powers of 2 (32x32, 64x64, 128x128, etc.)
- Use texture atlases for better performance

## Next Steps

- [Transform](transform.md) for positioning sprites
- [Tilemap](../tilemap/tilemap.md) for level backgrounds
- [Animation](animation.md) for animated sprites

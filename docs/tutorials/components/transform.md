# Transform Component

The `Transform` component defines an object's position, rotation, and scale in the game world.

## Adding Transform to an Object

```rust
use runa_core::components::Transform;

object.add_component(Transform::default());
```

This creates a transform with:
- Position: `(0, 0, 0)`
- Rotation: No rotation
- Scale: `(1, 1, 1)`

## Setting Position

```rust
use runa_core::glam::Vec3;

if let Some(transform) = object.get_component_mut::<Transform>() {
    // Set absolute position
    transform.position = Vec3::new(1.0, 2.0, 0.0);
    
    // Move relative to current position
    transform.position.x += 0.5;
    transform.position.y += 0.5;
}
```

## Rotation

```rust
if let Some(transform) = object.get_component_mut::<Transform>() {
    // Rotate around Z axis (2D rotation)
    transform.rotate_z(45.0); // 45 degrees
    
    // Rotate around X or Y axis (3D)
    transform.rotate_x(30.0);
    transform.rotate_y(60.0);
}
```

## Scale

```rust
if let Some(transform) = object.get_component_mut::<Transform>() {
    // Set scale
    transform.scale = Vec3::new(2.0, 2.0, 1.0); // 2x larger
}
```

## Complete Example: Moving Object

```rust
use runa_core::{
    components::Transform,
    ocs::{Object, Script},
    World,
    glam::Vec3,
};

pub struct Mover {
    speed: f32,
}

impl Mover {
    pub fn new() -> Self {
        Self { speed: 2.0 }
    }
}

impl Script for Mover {
    fn construct(&self, object: &mut Object) {
        object.add_component(Transform::default());
    }

    fn update(&mut self, object: &mut Object, dt: f32, _world: &mut World) {
        if let Some(transform) = object.get_component_mut::<Transform>() {
            // Move right at constant speed
            transform.position.x += self.speed * dt;
        }
    }
}
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `position` | `Vec3` | World position (x, y, z) |
| `rotation` | `Quat` | Rotation as quaternion |
| `scale` | `Vec3` | Scale factor (1, 1, 1 = normal size) |

## Tips

- Use `dt` (delta time) for frame-rate independent movement
- For 2D games, use Z rotation and keep Z position at 0
- Scale values less than 1.0 make objects smaller

## Next Steps

- [SpriteRenderer](sprite-renderer.md) to display images
- [Input](../systems/input.md) for player movement
- [Tilemap](tilemap.md) for level design

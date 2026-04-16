# Collision Components

Runa currently has **simple collision detection**, not a full 2D/3D physics engine.

There are currently two collision-related components in the codebase:

- `Collider2D`: simple AABB overlap checks for gameplay scripts
- `PhysicsCollision`: an existing collision-sized component still used by some runtime/editor paths

If you only need **2D overlap detection**, use `Collider2D`.

## Quick Start

```rust
use runa_core::components::Collider2D;

// Create an AABB collider (width, height)
let collider = Collider2D::new(32.0, 32.0);

// Add to object
object.add_component(collider);
```

## Creating Collision Boxes

```rust
// Create collision box
let mut collider = Collider2D::new(50.0, 100.0);

// Enable/disable collision
collider.enabled = true;  // Collisions active
collider.enabled = false; // Collisions ignored
```

## Checking Collisions

### Point in Collision Box

```rust
use runa_core::glam::Vec2;

if let Some(collider) = object.get_component::<Collider2D>() {
    if let Some(transform) = object.get_component::<Transform>() {
        let point = Vec2::new(10.0, 20.0);

        if collider.contains_point(point, transform.position.xy()) {
            println!("Point is inside collision box!");
        }
    }
}
```

### Collision Between Objects

```rust
fn check_collision(object1: &Object, object2: &Object) -> bool {
    let (transform1, collider1) = match (
        object1.get_component::<Transform>(),
        object1.get_component::<Collider2D>(),
    ) {
        (Some(t), Some(c)) => (t, c),
        _ => return false,
    };

    let (transform2, collider2) = match (
        object2.get_component::<Transform>(),
        object2.get_component::<Collider2D>(),
    ) {
        (Some(t), Some(c)) => (t, c),
        _ => return false,
    };

    collider1.intersects(
        transform1.position.xy(),
        collider2,
        transform2.position.xy(),
    )
}
```

## Complete Example: Player with Collision Query

```rust
use runa_core::{
    components::{Collider2D, Transform},
    input_system::*,
    ocs::{Object, Script},
    glam::Vec3,
};

pub struct Player {
    speed: f32,
}

impl Player {
    pub fn new() -> Self {
        Self { speed: 5.0 }
    }
}

impl Script for Player {
    fn construct(&self, object: &mut Object) {
        object
            .add_component(Transform::default())
            .add_component(Collider2D::new(32.0, 32.0));
    }

    fn update(&mut self, object: &mut Object, dt: f32) {
        let Some(current_position) = object
            .get_component::<Transform>()
            .map(|transform| transform.position)
        else {
            return;
        };

        let mut direction = Vec3::ZERO;
        if Input::is_key_pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if Input::is_key_pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if Input::is_key_pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if Input::is_key_pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        let movement = direction.normalize_or_zero() * self.speed * dt;
        let next_position = current_position + movement;

        if !object.would_collide_2d_at(next_position.xy()) {
            if let Some(transform) = object.get_component_mut::<Transform>() {
                transform.position = next_position;
            }
        }
    }
}
```

## Properties

### `Collider2D`

| Property     | Type   | Description                           |
| ------------ | ------ | ------------------------------------- |
| `half_size`  | `Vec2` | Half-size (extents) of the AABB       |
| `enabled`    | `bool` | Are overlap checks active             |
| `is_trigger` | `bool` | Reserved flag for trigger-style usage |

## Methods

```rust
// Create collision box (size is halved internally)
let collider = Collider2D::new(64.0, 64.0);
// Internal half_size will be (32.0, 32.0)

// Check if point is inside
let point = Vec2::new(10.0, 10.0);
let center = Vec2::new(0.0, 0.0);
if collider.contains_point(point, center) {
    // Point is inside
}
```

## Tips

- `Collider2D` is detection-only; it does not resolve movement or apply physics
- The size you pass to `Collider2D::new(width, height)` is halved internally
- For a 64x64 object, use `Collider2D::new(64.0, 64.0)`
- Disable overlap checks with `enabled = false` instead of removing the component
- `PhysicsCollision` still exists in the codebase, but `Collider2D` is the simpler script-facing API

## Next Steps

- [Transform](transform.md) for object positioning
- [Tilemap](../tilemap/tilemap.md) for tile-based collisions
- [Input](../systems/input.md) for movement controls

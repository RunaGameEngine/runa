# PhysicsCollision Component

The `PhysicsCollision` component defines a collision area for an object.

## Quick Start

```rust
use runa_core::components::PhysicsCollision;

// Create a collision box (width, height)
let collision = PhysicsCollision::new(32.0, 32.0);

// Add to object
object.add_component(collision);
```

## Creating Collision Boxes

```rust
// Create collision box
let mut collision = PhysicsCollision::new(50.0, 100.0);

// Enable/disable collision
collision.enabled = true;  // Collisions active
collision.enabled = false; // Collisions ignored
```

## Checking Collisions

### Point in Collision Box

```rust
use runa_core::glam::Vec2;

if let Some(collision) = object.get_component::<PhysicsCollision>() {
    if let Some(transform) = object.get_component::<Transform>() {
        let point = Vec2::new(10.0, 20.0);
        
        if collision.contains_point(point, transform.position.xy()) {
            println!("Point is inside collision box!");
        }
    }
}
```

### Collision Between Objects

```rust
fn check_collision(object1: &Object, object2: &Object) -> bool {
    let (transform1, collision1) = match (
        object1.get_component::<Transform>(),
        object1.get_component::<PhysicsCollision>(),
    ) {
        (Some(t), Some(c)) => (t, c),
        _ => return false,
    };
    
    let (transform2, collision2) = match (
        object2.get_component::<Transform>(),
        object2.get_component::<PhysicsCollision>(),
    ) {
        (Some(t), Some(c)) => (t, c),
        _ => return false,
    };
    
    // Simple AABB collision check
    let pos1 = transform1.position.xy();
    let pos2 = transform2.position.xy();
    
    let dx = (pos1.x - pos2.x).abs();
    let dy = (pos1.y - pos2.y).abs();
    
    let min_dx = collision1.size.x + collision2.size.x;
    let min_dy = collision1.size.y + collision2.size.y;
    
    dx < min_dx && dy < min_dy
}
```

## Complete Example: Player with Collision

```rust
use runa_core::{
    components::{PhysicsCollision, Transform},
    input_system::*,
    ocs::{Object, Script},
    World,
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
            .add_component(PhysicsCollision::new(32.0, 32.0));
    }

    fn update(&mut self, object: &mut Object, dt: f32, _world: &mut World) {
        if let Some(transform) = object.get_component_mut::<Transform>() {
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

            if direction.length() > 0.0 {
                direction = direction.normalize();
            }

            // Store old position for collision resolution
            let old_pos = transform.position;
            
            // Move
            transform.position += direction * self.speed * dt;
            
            // TODO: Check collision and resolve if needed
            // If collision detected, restore old_pos or slide along wall
        }
    }
}
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `size` | `Vec2` | Half-size (extents) of collision box |
| `enabled` | `bool` | Are collisions active |

## Methods

```rust
// Create collision box (size is halved internally)
let collision = PhysicsCollision::new(64.0, 64.0);
// Internal size will be (32.0, 32.0)

// Check if point is inside
let point = Vec2::new(10.0, 10.0);
let center = Vec2::new(0.0, 0.0);
if collision.contains_point(point, center) {
    // Point is inside
}
```

## Tips

- The size you pass is **halved** internally (it stores extents, not full size)
- For a 64x64 pixel object, use `new(64.0, 64.0)`
- Disable collisions with `enabled = false` instead of removing the component
- Use simple shapes (rectangles) for better performance

## Next Steps

- [Transform](transform.md) for object positioning
- [Tilemap](../tilemap/tilemap.md) for tile-based collisions
- [Input](../systems/input.md) for movement controls

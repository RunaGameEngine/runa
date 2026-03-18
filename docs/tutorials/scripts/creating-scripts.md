# Creating Scripts

Scripts are the primary way to add behavior to game objects in Runa Engine. This tutorial shows you how to create and use scripts.

## What is a Script?

A script is a Rust trait that defines how a game object behaves. Scripts have a lifecycle with three main methods:

1. **`construct()`** - Called once when the object is created
2. **`start()`** - Called once when the object enters the world
3. **`update()`** - Called every frame while the object exists

## Creating a Basic Script

### Step 1: Define Your Script Struct

```rust
use runa_core::{
    ocs::Script,
    ocs::Object,
    World,
};

pub struct MyScript {
    speed: f32,
}

impl MyScript {
    pub fn new() -> Self {
        Self { speed: 1.0 }
    }
}
```

### Step 2: Implement the Script Trait

```rust
impl Script for MyScript {
    /// Called once when the object is created
    fn construct(&self, object: &mut Object) {
        // Add components to the object
        // This happens before the object enters the world
    }

    /// Called once on the first frame
    fn start(&mut self, object: &mut Object) {
        // Initialize state, access other objects
        // This is called after the object is in the world
    }

    /// Called every frame
    fn update(&mut self, object: &mut Object, dt: f32, world: &mut World) {
        // Game logic goes here
        // dt = delta time in seconds
    }
}
```

## Complete Example: Rotating Sprite

```rust
use runa_core::{
    components::{SpriteRenderer, Transform},
    ocs::{Object, Script},
    World,
    glam::Vec3,
};

pub struct RotatingSprite {
    rotation_speed: f32,
}

impl RotatingSprite {
    pub fn new() -> Self {
        Self { rotation_speed: 90.0 } // 90 degrees per second
    }
}

impl Script for RotatingSprite {
    fn construct(&self, object: &mut Object) {
        // Add components
        object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::load_image!("assets/sprite.png")),
            });
    }

    fn start(&mut self, object: &mut Object) {
        // Set initial position
        if let Some(transform) = object.get_component_mut::<Transform>() {
            transform.position = Vec3::new(0.0, 0.0, 0.0);
        }
    }

    fn update(&mut self, object: &mut Object, dt: f32, _world: &mut World) {
        // Rotate the object
        if let Some(transform) = object.get_component_mut::<Transform>() {
            transform.rotate_z(self.rotation_speed * dt);
        }
    }
}
```

## Adding Your Script to the World

```rust
fn main() {
    let mut world = World::default();
    
    // Spawn your script
    world.spawn(Box::new(RotatingSprite::new()));
    
    // ... rest of setup
}
```

## Accessing Components

Scripts can get and modify components on their object:

```rust
fn update(&mut self, object: &mut Object, dt: f32, world: &mut World) {
    // Get a component (returns Option)
    if let Some(transform) = object.get_component::<Transform>() {
        println!("Position: {:?}", transform.position);
    }

    // Get a mutable component
    if let Some(transform) = object.get_component_mut::<Transform>() {
        transform.position.x += 1.0 * dt;
    }
}
```

## Lifecycle Summary

| Method | When Called | Use For |
|--------|-------------|---------|
| `construct()` | Once, before object enters world | Adding components |
| `start()` | Once, on first frame | Initialization, finding other objects |
| `update()` | Every frame | Game logic, movement, input |

## Next Steps

- Learn about [Transform](../components/transform.md) for position and rotation
- Explore [Input](../systems/input.md) for player controls
- Check out [Audio](../systems/audio.md) for playing sounds

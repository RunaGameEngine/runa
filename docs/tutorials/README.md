# Runa Engine Tutorials

Welcome to the Runa Engine tutorials! These guides will help you learn how to build games with Runa Engine.

## Getting Started

New to Runa Engine? Start here:

### Quick Start Guides

1. [Creating a 2D Game](getting-started/creating-a-2d-game.md) - Complete guide for 2D games with player movement and mouse interaction
2. [Creating a 3D Game](getting-started/creating-a-3d-game.md) - Complete guide for 3D games with FPS camera and mesh rendering
3. [Creating Your First App](getting-started/creating-your-first-app.md) - Minimal application setup
4. [Creating Scripts](scripts/creating-scripts.md) - Add behavior to game objects
5. [Input System](systems/input.md) - Handle keyboard, mouse, and window control

## Choosing Your Path

### 2D Game Development

If you're making a 2D game (platformer, top-down, puzzle):

1. Start with [Creating a 2D Game](getting-started/creating-a-2d-game.md)
2. Learn about [Tilemaps](tilemap/tilemap.md) for level design
3. Add interactivity with [CursorInteractable](components/cursor-interactable.md)

### 3D Game Development

If you're making a 3D game (FPS, third-person, exploration):

1. Start with [Creating a 3D Game](getting-started/creating-a-3d-game.md)
2. Learn about the [Camera system](#camera) for different perspectives
3. Create custom meshes with vertices and indices

## Tutorials by Category

### Core Concepts

Fundamental building blocks:

- [Transform](components/transform.md) - Position, rotation, and scale
- [Scripts](scripts/creating-scripts.md) - Add behavior with the Script trait
- [Input](systems/input.md) - Keyboard, mouse, and cursor control

### Components

Components add properties and features to game objects:

#### Rendering

- [SpriteRenderer](components/sprite-renderer.md) - Display 2D images
- [MeshRenderer](components/mesh-renderer.md) - Display 3D meshes
- [TilemapRenderer](tilemap/tilemap.md) - Tile-based 2D levels

#### Camera

- [Camera](getting-started/creating-a-2d-game.md#understanding-the-camera) - Unified 2D/3D camera system
  - `Camera::new_ortho()` - For 2D games
  - `Camera::new_perspective()` - For 3D games
- [ActiveCamera](getting-started/creating-a-3d-game.md) - Mark the active camera

#### Interaction

- [CursorInteractable](components/cursor-interactable.md) - Mouse hover and click events

#### Collision

- [Collider2D](components/physics-collision.md) - Simple AABB overlap detection
- [PhysicsCollision](components/physics-collision.md) - Existing collision-sized component used by current editor/runtime paths

### Systems

Core engine systems:

- [Input](systems/input.md) - Keyboard, mouse, cursor, and window control
- [Audio](systems/audio.md) - Sound effects and music
- [Rendering](../architecture/renderer.md) - How rendering works

### Tilemaps

Create 2D levels:

- [Tilemap](tilemap/tilemap.md) - Complete tilemap guide

## Example Code

All examples assume you have the following imports:

```rust
use runa_engine::runa_core::{
    components::*,
    input_system::*,
    ocs::{Object, Script, World},
    glam::{Vec2, Vec3, Quat},
};
```

## Camera System

Runa Engine uses a unified `Camera` component for both 2D and 3D:

### 2D Orthographic Camera

```rust
object.add_component(Camera::new_ortho(
    32.0,        // width in world units
    18.0,        // height in world units
    (1280, 720)  // viewport size in pixels
));
```

### 3D Perspective Camera

```rust
object.add_component(Camera::new_perspective(
    position,           // Vec3 camera position
    target,             // Vec3 look-at point
    Vec3::Y,            // Up vector
    75.0.to_radians(), // Field of view
    0.1,                // Near clipping plane
    1000.0,             // Far clipping plane
    (1280, 720)         // Viewport size
));
```

## Getting Help

- Check the [architecture](../architecture/) documentation for deeper understanding
- Look at the [examples](../../examples/) folder for complete working projects
- Review the API documentation for detailed reference

## Next Steps

After completing these tutorials:

1. Experiment with the example projects (`sandbox`, `sandbox_3d`)
2. Combine components to create complex behaviors
3. Expect API changes while the engine is still pre-alpha

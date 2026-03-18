# Runa Engine Tutorials

Welcome to the Runa Engine tutorials! These guides will help you learn how to build games with Runa Engine.

## Getting Started

New to Runa Engine? Start here:

1. [Creating Your First App](getting-started/creating-your-first-app.md) - Set up a basic application
2. [Creating Scripts](scripts/creating-scripts.md) - Add behavior to game objects
3. [Input System](systems/input.md) - Handle keyboard and mouse input

## Tutorials by Category

### Components

Components add properties and features to game objects:

- [Transform](components/transform.md) - Position, rotation, and scale
- [SpriteRenderer](components/sprite-renderer.md) - Display images
- [CursorInteractable](components/cursor-interactable.md) - Mouse hover and click events
- [PhysicsCollision](components/physics-collision.md) - Collision detection

### Systems

Core engine systems:

- [Input](systems/input.md) - Keyboard and mouse input
- [Audio](systems/audio.md) - Sound effects and music

### Tilemaps

Create 2D levels:

- [Tilemap](tilemap/tilemap.md) - Complete tilemap guide

## Example Code

All examples assume you have the following imports:

```rust
use runa_core::{
    components::*,
    input_system::*,
    ocs::{Object, Script, World},
    glam::Vec3,
};
```

## Getting Help

- Check the [architecture](../architecture/) documentation for deeper understanding
- Look at the [examples](../../examples/) folder for complete working projects
- Review the API documentation for detailed reference

## Next Steps

After completing these tutorials:

1. Experiment with the example projects
2. Combine components to create complex behaviors
3. Build your own game!

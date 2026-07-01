# Creating Your First App

This tutorial shows the smallest useful Runa application using the code-first API.

## Dependency

```toml
[dependencies]
runa_engine = { git = "https://github.com/RunaGameEngine/runa.git", tag = "v0.6.0-alpha.1" }
```

## Minimal App

```rust
use runa_engine::prelude::*;
use runa_engine::runa_app::{RunaApp, RunaWindowConfig};

fn main() {
    let mut world = World::new();

    // Spawn an object with a Transform and nothing else
    world.spawn_bundle((Transform::default(),));

    let config = RunaWindowConfig {
        title: "My First Game".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: true,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(world, config);
}
```

## Adding Components

Define a component with `#[derive(Component)]` — no registration required:

```rust
use runa_engine::prelude::*;

#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Component)]
struct Player;
```

Then spawn an object with those components:

```rust
fn setup(world: &mut World) {
    world.spawn_bundle((
        Transform::default(),
        Player,
        Health { current: 100, max: 100 },
    ));
}
```

## Adding Script Behavior

Implement `Script` on your component to give it lifecycle methods:

```rust
use runa_engine::prelude::*;

#[derive(Component)]
struct PlayerController {
    speed: f32,
}

impl Script for PlayerController {
    fn start(&mut self, ctx: &mut ScriptContext) {
        println!("PlayerController started on {:?}", ctx.id());
    }

    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position += Vec3::X * self.speed * dt;
        }
    }
}
```

Then include it in a spawn bundle:

```rust
world.spawn_bundle((
    Transform::default(),
    SpriteRenderer::new(None),
    PlayerController { speed: 0.25 },
));
```

## Named Objects

For objects with a specific name (useful for debugging and lookup):

```rust
use runa_engine::runa_core::ocs::Object;

let id = world.spawn_object(
    Object::new("Player")
        .with(Transform::default())
        .with(SpriteRenderer::new(None))
        .with(PlayerController { speed: 0.25 }),
);
```

## Querying Objects

Find all objects with a specific component:

```rust
for id in world.find_all_with::<Player>() {
    let health = world.get::<Health>(id).unwrap();
    println!("Player {:?} has {} HP", id, health.current);
}
```

Find the first matching object:

```rust
if let Some(id) = world.find_first_with::<ActiveCamera>() {
    println!("Found active camera: {:?}", id);
}
```

## Deferred Commands

Mutate the world from within a script using deferred commands:

```rust
impl Script for Enemy {
    fn update(&mut self, ctx: &mut ScriptContext, _dt: f32) {
        if self.health <= 0 {
            if let Some(id) = ctx.id() {
                ctx.commands().despawn(id);
            }
        }
    }
}
```

Commands are applied after the update phase, so iteration is never invalidated mid-frame.

## Full Example

```rust
use runa_engine::prelude::*;
use runa_engine::runa_app::{RunaApp, RunaWindowConfig};
use runa_engine::runa_core::components::*;

#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Component)]
struct PlayerController {
    speed: f32,
}

impl Script for PlayerController {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position += Vec3::X * self.speed * dt;
        }
    }
}

fn main() {
    let mut world = World::new();

    world.spawn_bundle((
        Transform::default(),
        Camera::new_orthographic(320.0, 180.0),
        ActiveCamera,
        SpriteRenderer::new(None),
        PlayerController { speed: 0.25 },
    ));

    let config = RunaWindowConfig {
        title: "My Game".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: true,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(world, config);
}
```

## Summary

- Define components with `#[derive(Component)]` — no registration
- Spawn objects with `world.spawn_bundle((...))` or `world.spawn_object(Object::new("name").with(...))`
- Add behavior by implementing `Script` on your component
- Query with `world.find_all_with::<T>()` and `world.find_first_with::<T>()`
- Mutate the world from scripts via `ctx.commands()`

## Next Steps

- [Creating Scripts](../scripts/creating-scripts.md)
- [Creating a 2D Game](creating-a-2d-game.md)
- [Creating a 3D Game](creating-a-3d-game.md)

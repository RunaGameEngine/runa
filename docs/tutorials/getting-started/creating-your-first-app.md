# Creating Your First App

This tutorial shows the smallest useful Runa application using the current runtime model.

## Dependency

```toml
[dependencies]
runa_engine = { git = "https://github.com/RunaGameEngine/runa.git", tag = "v0.5.1-alpha.1" }
```

## Minimal App

```rust
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine, RunaArchetype,
};
use runa_engine::runa_core::ocs::{Object, World};

#[derive(RunaArchetype)]
struct EmptyArchetype;

impl EmptyArchetype {
    fn create(world: &mut World) -> u64 {
        world.spawn(Object::new("Empty"))
    }
}

fn main() {
    let mut engine = Engine::new();
    engine.register_archetype::<EmptyArchetype>();

    let mut world = engine.create_world();
    let _ = world.spawn_archetype::<EmptyArchetype>();

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

## What Is Happening

- `Engine` holds explicit runtime registration state
- `register_archetype::<T>()` registers a typed code-first object factory
- `create_world()` returns a `World` that knows about that registry
- `spawn_archetype::<T>()` instantiates the object and gives it an `ObjectId`

You can still build objects manually and call `world.spawn(...)` directly. The engine/bootstrap layer is optional, but it is now the recommended place for:

- type registration
- archetype registration
- future serialization/editor bootstrap hooks

## Manual Spawn Variant

```rust
use runa_engine::runa_core::ocs::Object;

let mut world = Engine::new().create_world();
let id = world.spawn(Object::new("Manual Object"));
let object = world.get(id);
```

## Notes

- `Object::new(...)` already includes a default `Transform`
- scripts are attached with `.with(MyScript::new())`
- scripts do not construct objects
- world mutations during script update should go through `ctx.commands()`

## Next Steps

- [Creating Scripts](../scripts/creating-scripts.md)
- [Creating a 2D Game](creating-a-2d-game.md)
- [Creating a 3D Game](creating-a-3d-game.md)

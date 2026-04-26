# Runa Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

<div align="center">
  <img src="TheRunaGameEngine.png" alt="https://github.com/RunaGameEngine/runa">
</div>

Runa Engine is an experimental Rust game engine workspace built around a code-first runtime, a `wgpu` renderer, project tooling, and an optional editor.

> Status: pre-alpha. APIs are still evolving. The runtime is usable for prototypes and internal tools, but the engine is not production-ready yet.

## What Runa Is

Runa is a workspace, not just one crate. It currently includes:

- `runa_core`: world, objects, components, scripts, input, audio
- `runa_app`: runtime app loop and window bootstrap
- `runa_render`: `wgpu` renderer
- `runa_asset`: asset loading helpers
- `runa_project`: project and world serialization/scaffolding
- `runa_editor`: optional editor
- `runa_hub`: optional launcher/project hub
- `runa_engine`: umbrella crate for normal game-side usage

The runtime is code-first:

- `World` owns `Object`s
- `Object` is a component container with `ObjectId`
- `Transform` exists on every object by default
- scripts are attachable behavior components
- objects are composed explicitly in Rust code

## What Works Today

### Runtime

- object/component world model
- attachable script behaviors with `start()`, `update()`, and `late_update()`
- `ObjectId`-based lookup and simple queries
- deferred world commands from scripts
- archetype registration and spawning
- type metadata registration for components and scripts
- serialized field metadata for editor/tooling flows

### Rendering

- 2D sprite rendering
- tilemaps
- basic 3D mesh rendering
- unified camera component for 2D/3D
- offscreen render targets used by the editor

### Systems

- global input API
- window control from scripts
- basic audio and spatial audio
- simple 2D AABB collision detection with `Collider2D`
- cursor interaction

### Tooling

- `.runaproj` project manifest
- world save/load in RON
- project scaffolding
- experimental editor
- experimental hub/launcher
- editor-side build flow with project/build settings

## Current Limits

- single runtime window only
- no full physics engine
- no mature animation pipeline
- 3D support is still basic
- generic registry-driven serialization is not finished yet
- editor is optional and incomplete
- API stability is not guaranteed between alpha revisions

## Quick Start

### Requirements

- Rust 1.75+
- GPU/backend supported by `wgpu`

### Run a bundled example

```bash
cargo run -p sandbox
```

### Add Runa to a new project

Current latest public tag: [`v0.5.1-alpha.1`](https://github.com/RunaGameEngine/runa/releases/tag/v0.5.1-alpha.1)

```toml
[dependencies]
runa_engine = { git = "https://github.com/RunaGameEngine/runa.git", tag = "v0.5.1-alpha.1" }
```

If you want to track the repository head instead of a tag:

```toml
[dependencies]
runa_engine = { git = "https://github.com/RunaGameEngine/runa.git", branch = "main" }
```

## Quick Guide

Minimal startup:

```rust
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine, RunaArchetype,
};
use runa_engine::runa_core::ocs::{Object, World};

#[derive(RunaArchetype)]
#[runa(name = "player")]
struct PlayerArchetype;

impl PlayerArchetype {
    fn create(world: &mut World) -> u64 {
        world.spawn(Object::new("Player"))
    }
}

fn main() {
    let mut engine = Engine::new();
    engine.register_archetype::<PlayerArchetype>();

    let mut world = engine.create_world();
    let _ = world.spawn_archetype::<PlayerArchetype>();

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

Typical gameplay object:

```rust
use runa_engine::runa_asset::load_image;
use runa_engine::runa_core::{
    components::{ActiveCamera, Camera, SpriteRenderer},
    ocs::{Object, Script, ScriptContext},
    Vec3,
};

struct PlayerController {
    speed: f32,
}

impl PlayerController {
    fn new() -> Self {
        Self { speed: 0.25 }
    }
}

impl Script for PlayerController {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if let Some(transform) = ctx.get_component_mut::<runa_engine::runa_core::components::Transform>() {
            transform.position += Vec3::X * self.speed * dt;
        }
    }
}

fn create_player() -> Object {
    Object::new("Player")
        .with(Camera::new_ortho(320.0, 180.0))
        .with(ActiveCamera)
        .with(SpriteRenderer::new(Some(load_image!("assets/art/player.png"))))
        .with(PlayerController::new())
}
```

## How To Start Making A Game

1. Create an `Engine`.
2. Register your components/scripts/archetypes in one bootstrap function.
3. Create a `World` from the engine.
4. Spawn your archetypes or objects.
5. Run the app with `RunaApp::run_with_config(...)`.

Recommended project shape:

```text
src/
  main.rs
  player.rs
  enemy.rs
  world_setup.rs
assets/
```

Good practice in Runa:

- keep object composition in typed archetypes or explicit factory functions
- keep behavior in scripts
- use typed marker/data components instead of string tags
- use `ObjectId` and queries for object communication

Script fields intended for editor/runtime serialization must be marked explicitly with `#[serialize_field]` when using the derive macros. For script/component default values visible to the editor, the recommended path is `impl Default` on the type.

Editor workflow notes:

- `Project Settings` edits the project manifest, including the `RunaWindowConfig` values used by `Play In Window`
- `Build Settings` controls release/debug output and Windows console-hiding for final builds
- `Content Browser -> Live Rust` now separates `New Rust Script` from `New Rust Archetype`, so scripts can be authored without bundling archetype code into the same file

## Documentation

- [Tutorial Index](docs/tutorials/README.md)
- [Runtime And Editor Update Notes](docs/architecture/runtime-and-editor-update.md)
- [Creating Your First App](docs/tutorials/getting-started/creating-your-first-app.md)
- [Creating a 2D Game](docs/tutorials/getting-started/creating-a-2d-game.md)
- [Creating a 3D Game](docs/tutorials/getting-started/creating-a-3d-game.md)
- [Creating Scripts](docs/tutorials/scripts/creating-scripts.md)
- [Transform](docs/tutorials/components/transform.md)
- [Input](docs/tutorials/systems/input.md)
- [Object Model Notes](docs/architecture/object-model.md)
- [Registration And Archetypes](docs/tutorials/advanced/registration-and-archetypes.md)

## Repository

- GitHub: <https://github.com/RunaGameEngine/runa>
- Releases: <https://github.com/RunaGameEngine/runa/releases>
- Tags: <https://github.com/RunaGameEngine/runa/tags>

## License

Dual-licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

# Runa Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

**Runa Engine** is an experimental Rust game engine workspace focused on a small script-driven runtime, a `wgpu` renderer, and in-repo tooling for project creation and scene editing.

> **Status:** Pre-Alpha. APIs are unstable, tooling is still evolving, and the engine is not ready for production use.

## Current State

Runa is currently a **single-window runtime** with:

- 2D sprite rendering
- Tilemaps
- Basic 3D mesh rendering
- Script-driven objects
- Global input API
- Basic audio and spatial audio
- World/project serialization in RON
- Experimental editor and hub applications

The repository is a workspace, not just a runtime crate. It contains runtime, rendering, assets, project scaffolding, editor, and launcher tools.

## What Works Today

### Runtime

- `World`, `Object`, and `Script` lifecycle (`construct`, `start`, `update`)
- `Transform`, `SpriteRenderer`, `MeshRenderer`, `Tilemap`, `Camera`, `AudioSource`
- Unified 2D/3D camera component
- Cursor interaction via `CursorInteractable`
- Simple 2D AABB overlap detection via `Collider2D`
- Window control from scripts:
  - change window title
  - toggle/set fullscreen
  - resize the main window
  - move the main window on screen
  - query the center of the current monitor
  - center the window on the current monitor

### Rendering

- 2D textured sprites
- Tilemap rendering with negative coordinates
- Basic 3D mesh path with depth buffer
- Offscreen render target support used by the editor

### Tooling

- `.runaproj` project manifest
- world save/load in RON
- project scaffolding through `runa_project`
- `runa_editor` for scene inspection/editing
- `runa_hub` for creating/opening projects

## Current Limitations

- Single runtime window only
- No full physics engine
- No mature animation system
- No stable shader/material pipeline for user-defined shaders
- 3D support is still basic and experimental
- Editor and hub are usable but not feature-complete
- API stability is not guaranteed between alpha builds

## Workspace Layout

```text
runa-engine/
├── crates/
│   ├── runa_app/         # Runtime app loop and window setup
│   ├── runa_asset/       # Asset loading helpers
│   ├── runa_core/        # World, components, scripts, input, audio
│   ├── runa_editor/      # Experimental editor
│   ├── runa_engine/      # Umbrella re-export crate
│   ├── runa_hub/         # Experimental launcher/project hub
│   ├── runa_project/     # Project manifests, world serialization, scaffolding
│   ├── runa_render/      # wgpu renderer
│   └── runa_render_api/  # Render command layer
├── docs/
├── examples/
│   ├── sandbox/
│   ├── sandbox_3d/
│   └── sandbox_soundtest/
├── CHANGELOG.md
└── Cargo.toml
```

## Quick Start

### Requirements

- Rust 1.75+
- A GPU/backend supported by `wgpu`

### Run a bundled example

```bash
cargo run -p sandbox
```

### Use the umbrella crate

```toml
[dependencies]
runa_engine = { git = "https://github.com/AnuranGames/runa-engine.git", tag = "v0.2.0-alpha.2" }
```

## Minimal 2D Example

```rust
use runa_engine::runa_app::{RunaApp, RunaWindowConfig};
use runa_engine::runa_core::{
    components::{ActiveCamera, Camera, SpriteRenderer, Transform},
    input_system::*,
    ocs::Script,
    Vec3, World,
};

fn main() {
    let mut world = World::default();
    world.spawn(Box::new(Player::new()));

    let config = RunaWindowConfig {
        title: "My 2D Game".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: true,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(world, config);
}

struct Player {
    speed: f32,
}

impl Player {
    fn new() -> Self {
        Self { speed: 0.25 }
    }
}

impl Script for Player {
    fn construct(&self, object: &mut runa_engine::runa_core::ocs::Object) {
        object
            .add_component(Transform::default())
            .add_component(Camera::new_ortho(320.0, 180.0, (1280, 720)))
            .add_component(ActiveCamera)
            .add_component(SpriteRenderer::new(Some(
                runa_engine::runa_asset::load_image!("assets/Charactert.png"),
            )));
    }

    fn update(&mut self, object: &mut runa_engine::runa_core::ocs::Object, _dt: f32) {
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
            transform.position += direction.normalize_or_zero() * self.speed;
        }
    }
}
```

## Script Window Control Example

```rust
use runa_engine::runa_core::input_system::*;

if Input::is_key_just_pressed(KeyCode::F1) {
    set_window_title("Debug Mode");
}

if Input::is_key_just_pressed(KeyCode::F2) {
    toggle_fullscreen();
}

if Input::is_key_just_pressed(KeyCode::F3) {
    set_window_size(1600, 900);
}

if Input::is_key_just_pressed(KeyCode::F4) {
    center_window();
}

if Input::is_key_pressed(KeyCode::ArrowRight) {
    move_window_by(8, 0);
}
```

## Documentation

- [Tutorial Index](docs/tutorials/README.md)
- [Create a 2D Game](docs/tutorials/getting-started/creating-a-2d-game.md)
- [Create a 3D Game](docs/tutorials/getting-started/creating-a-3d-game.md)
- [Input System](docs/tutorials/systems/input.md)
- [Renderer Notes](docs/architecture/renderer.md)

## License

Dual-licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

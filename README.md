# Runa Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

**Runa Engine** — An experimental 2D/3D game engine written in Rust, focused on performance and developer ergonomics.

> ⚠️ **Status**: Early development (Pre-Alpha). API is unstable. Not for production use.

## 🌟 Features

### ✅ Implemented

- **2D Rendering**
  - Sprites with textures and rotation
  - Automatic batching (1000+ objects = 1 draw call)
  - Transparency (alpha blending)
  - Tilemap with negative coordinate support
- **Object Component System (OCS)**
  - `Transform` component (mandatory for all objects)
  - Scripts via `Script` trait
  - Global input access (`Input::is_key_pressed()`)
- **Input System**
  - Keyboard and mouse handling
  - Screen-to-world coordinate conversion
  - `CursorInteractable` component for object clicks

### 🚧 In Progress

- [ ] World (Scene) serialization
- [ ] Object serialization
- [ ] Physics (2D/3D)
- [ ] Animations (sprites, skeletal)
- [ ] Level editor
- [ ] Custom shader support
- [ ] Tilemap
- [ ] **Audio** (basic)
  - [ ] Sound playback via `rodio`
- [ ] **3D Support** (experimental)
  - [ ] Mesh pipeline with depth buffer
  - [ ] Instancing for massive object rendering
  - [ ] Basic lighting (diffuse + ambient)

## 🚀 Quick Start

### Requirements

- Rust 1.75+
- GPU with Vulkan/Metal/DirectX 12 support

### Run Example

```bash
# Clone (when public)
git clone https://github.com/AnuranGames/runa-engine
cd runa-engine

# Run sandbox example
cargo run --example sandbox
```

### Create a new game project with Runa:

    Create project:

```sh
cargo new my_game
cd my_game
```

    Add dependencies:

```toml
[dependencies]
runa_engine = { git = "https://github.com/AnuranGames/runa-engine.git", tag = "v0.1.0-alpha.1" }
```

### Create Your Game with Player Script

```rust
// main.rs
use runa_app::{RunaApp, RunaWindowConfig};
use runa_core::World;

use runa_core::Vec3;
use runa_core::{
    components::{SpriteRenderer, Transform},
    input_system::*,
    ocs::Script,
};

fn main() {
    // Create a new empty world to hold game objects and systems
    let mut world = World::default();

    // Spawn the player object (managed via its Script implementation)
    world.spawn(Box::new(Player::new()));

    // Configure the application window
	let config = RunaWindowConfig {
		title: "Sandbox".to_string(),
		width: 1280,
		height: 720,
		fullscreen: false,
		vsync: true,
	};

	// Launch the engine with the configured world and window settings
	let _ = RunaApp::run_with_config(world, config);
	// or run with default window settings
	let _ = RunaApp::run_default(world);
}

/// Player script — defines behavior for the player-controlled character.
pub struct Player {
    speed: f32,
    direction: Vec3,
}

impl Player {
    pub fn new() -> Self {
        Self {
            speed: 0.25,
            direction: Vec3::ZERO,
        }
    }
}

impl Script for Player {
    /// Called once when the object is created.
    /// Initializes components (transform + sprite).
    fn construct(&self, _object: &mut runa_core::ocs::Object) {
        _object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::loader::load_image("assets/Charactert.png")),
            });
    }

    /// Called once on the first tick after the object is added to the world.
    /// Sets initial position and scale.
    fn start(&mut self, _object: &mut runa_core::ocs::Object) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            transform.position = Vec3::new(0.0, 0.0, 0.0);
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        }
    }

    /// Called every tick. Handles input and updates position.
    fn update(&mut self, _object: &mut runa_core::ocs::Object, _dt: f32) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            // Reset movement direction
            self.direction = Vec3::ZERO;

            // Read input state (WASD keys)
            if Input::is_key_pressed(KeyCode::KeyW) {
                self.direction.y = 1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyS) {
                self.direction.y = -1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyD) {
                self.direction.x = 1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyA) {
                self.direction.x = -1.0;
            }

            // Apply normalized movement (diagonal speed compensation)
            transform.position += self.direction.normalize_or_zero() * self.speed;
        }
    }
}


```

## 📂 Project Structure

```
runa-engine/
├── crates/
│   ├── runa_app/           # App entrypoint (RunaApp and WindowConfig)
│   ├── runa_assets/        # Audio system (rodio)
│   ├── runa_core/          # Core: ECS, components, scripts
│   ├── runa_editor/        # Editor and debugger for designing Runa Engine games
│   ├── runa_hub/           # Launcher for creating/managing projects
│   ├── runa_render/        # wgpu renderer
│   └── runa_render_api/    # Renderer-agnostic commands
├── examples/             # Dev tests
│   └── sandbox/            # Test sandbox
├── docs/                 # Documentation
├── CHANGELOG.md          # Changelog
├── README.md             # This file
└── Cargo.toml            # Workspace root
```

## 📜 License

Dual-licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

Choose whichever suits your project best.

## 🤝 Contributing

Project is currently private. When public:

- Open Issues for bugs and feature requests
- Submit PRs to `dev` branch
- Follow Conventional Commits

## 🙏 Acknowledgements

- **wgpu** — Cross-platform graphics API
- **glam** — Math library
- **rodio** — Audio playback

---

✨ _Built with ❤️ in Rust_ ✨

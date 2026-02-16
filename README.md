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

### Create Your Player Script with simple controller
```rust
use glam::Vec3;
use runa_core::{
    components::{SpriteRenderer, Transform},
    input_system::*,
    ocs::Script,
};

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
    fn construct(&self, _object: &mut runa_core::ocs::Object) {
        // конструктор объекта
        _object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(
                runa_asset::loader::load_image("assets/Charactert.png")
                ),
            });
    }

    fn start(&mut self, _object: &mut runa_core::ocs::Object) {}

    fn update(&mut self, _object: &mut runa_core::ocs::Object, _dt: f32) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            self.direction = Vec3::ZERO;
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
            transform.position += self.direction.normalize_or_zero() * self.speed;
        }
    }
}

```

## 📂 Project Structure
```
runa-engine/
├── crates/
│   └── runa-assets/        # Audio system (rodio)
│   ├── runa-core/          # Core: ECS, components, scripts
│   ├── runa-editor/        # Editor and debugger for designing Runa Engine games
│   ├── runa-hub/           # Launcher for creating/managing projects
│   ├── runa-render/        # wgpu renderer
│   ├── runa-render-api/    # Renderer-agnostic commands
├── examples/
│   └── sandbox/            # Test sandbox
├── docs/                   # Documentation
├── CHANGELOG.md            # Changelog
├── README.md               # This file
└── Cargo.toml              # Workspace root
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

✨ *Built with ❤️ in Rust* ✨
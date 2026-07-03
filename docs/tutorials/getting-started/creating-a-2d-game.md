# Creating a 2D Game

This guide shows the current recommended Runa pattern for a small 2D game.

## Main File

```rust
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine,
};

mod player;

fn main() {
    let engine = Engine::new();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();

        let _ = world.spawn_object(player::new_player());
    } // <- Drop world.

    let config = RunaWindowConfig {
        title: "My 2D Game".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: true,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(world_rc, config);
}
```

## Player Module

```rust
use runa_engine::{Engine, Component};
use runa_engine::runa_asset::load_image;
use runa_engine::runa_core::{
    components::{ActiveCamera, Camera, Collider2D, SpriteRenderer, Transform},
    glam::Vec3,
    input_system::*,
    ocs::{Object, Script, ScriptContext, World},
};

#[derive(Component)]
pub struct Health {
    pub current: i32,
}

// <- No need to add #[derive(Component)] if you implement Script.
pub struct PlayerController {
    speed: f32,
}

impl PlayerController {
    pub fn new() -> Self {
        Self { speed: 0.25 }
    }
}

impl Script for PlayerController { // <- here
    fn start(&mut self, ctx: &mut ScriptContext) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position = Vec3::ZERO;
        }
    }

    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        let mut direction = Vec3::ZERO;

        if InputState::is_key_pressed(KeyCode::KeyW) {
            direction.y = 1.0;
        }
        if InputState::is_key_pressed(KeyCode::KeyS) {
            direction.y = -1.0;
        }
        if InputState::is_key_pressed(KeyCode::KeyD) {
            direction.x = 1.0;
        }
        if InputState::is_key_pressed(KeyCode::KeyA) {
            direction.x = -1.0;
        }

        let Some(current_position) = ctx
            .get_component::<Transform>()
            .map(|transform| transform.position)
        else {
            return;
        };

        let movement = self.direction.normalize_or_zero() * self.speed * _dt;
        let next_position = current_position + movement;

        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position = next_position;
        }
    }
}


pub fn new_player() -> Object {
    Object::new("Player")
        .with(Camera::new_orthographic(320.0, 180.0))
        .with(ActiveCamera)
        .with(SpriteRenderer::new(Some(load_image!("assets/art/player.png"))))
        .with(Collider2D::new(16.0, 16.0))
        .with(Health { current: 100 })
        .with(PlayerController::new())
}
```

## Why It Looks Like This

The player object is assembled explicitly in a factory function.

The script does not add its own required components anymore. That makes:

- object shape visible at spawn time
- runtime behavior easier to reason about
- factory functions reusable from both code and future editor tooling

## Next Steps

- [Creating Scripts](../scripts/creating-scripts.md)
- [Collider2D](../components/physics-collision.md)
- [Tilemap](../tilemap/tilemap.md)

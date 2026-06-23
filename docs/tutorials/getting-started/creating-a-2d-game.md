# Creating a 2D Game

This guide shows the current recommended Runa pattern for a small 2D game:

- typed archetypes for reusable objects
- attachable script behavior
- explicit bootstrap registration
- typed archetype spawning

## Main File

```rust
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine,
};

mod player;

fn register_game_types(engine: &mut Engine) {
    player::register_types(engine);
}

fn main() {
    let mut engine = Engine::new();
    register_game_types(&mut engine);

    let world_rc = engine.create_world();
    world_rc.borrow_mut().spawn_archetype::<player::PlayerArchetype>();

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
use runa_engine::{Engine, RunaArchetype, RunaComponent, RunaScript};
use runa_engine::runa_asset::load_image;
use runa_engine::runa_core::{
    components::{ActiveCamera, Camera, Collider2D, SpriteRenderer, Transform},
    glam::Vec3,
    input_system::*,
    ocs::{Object, Script, ScriptContext, World},
};

#[derive(RunaComponent)]
pub struct Health {
    pub current: i32,
}

#[derive(RunaScript)]
pub struct PlayerController {
    speed: f32,
}

impl PlayerController {
    pub fn new() -> Self {
        Self { speed: 0.25 }
    }
}

impl Script for PlayerController {
    fn start(&mut self, ctx: &mut ScriptContext) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position = Vec3::ZERO;
        }
    }

    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        let mut direction = Vec3::ZERO;

        if Input::is_key_pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if Input::is_key_pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }

        let Some(current_position) = ctx.get_component::<Transform>().map(|t| t.position) else {
            return;
        };

        let next_position = current_position + direction.normalize_or_zero() * self.speed * dt;
        if !ctx.would_collide_2d_at(next_position.truncate()) {
            if let Some(transform) = ctx.get_component_mut::<Transform>() {
                transform.position = next_position;
            }
        }
    }
}

#[derive(RunaArchetype)]
#[runa(name = "player")]
pub struct PlayerArchetype;

impl PlayerArchetype {
    pub fn create(world: &mut World) -> u64 {
        world.spawn(
            Object::new("Player")
                .with(Camera::new_orthographic(320.0, 180.0))
                .with(ActiveCamera)
                .with(SpriteRenderer::new(Some(load_image!("assets/art/player.png"))))
                .with(Collider2D::new(16.0, 16.0))
                .with(Health { current: 100 })
                .with(PlayerController::new())
        )
    }
}

pub fn register_types(engine: &mut Engine) {
    engine.register::<Health>();
    engine.register::<PlayerController>();
    engine.register_archetype::<PlayerArchetype>();
}
```

## Why It Looks Like This

The player object is assembled explicitly in `PlayerArchetype::create(...)`.

The script does not add its own required components anymore. That makes:

- object shape visible at registration/spawn time
- runtime behavior easier to reason about
- archetypes reusable from both code and future editor tooling

## Next Steps

- [Creating Scripts](../scripts/creating-scripts.md)
- [Collider2D](../components/physics-collision.md)
- [Tilemap](../tilemap/tilemap.md)

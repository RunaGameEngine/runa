# Runa Engine Tutorials

These guides assume the current runtime model:

- `Object` is a component container with identity
- `Transform` exists by default
- scripts are behavior attachments
- world mutations from scripts use deferred commands
- code-first bootstrap stays first-class
- registration and archetypes are runtime-owned, not editor-owned

## Getting Started

1. [Creating Your First App](getting-started/creating-your-first-app.md)
2. [Creating a 2D Game](getting-started/creating-a-2d-game.md)
3. [Creating a 3D Game](getting-started/creating-a-3d-game.md)
4. [Creating Scripts](scripts/creating-scripts.md)
5. [Registration And Archetypes](advanced/registration-and-archetypes.md)

## Core Concepts

- [Scripts](scripts/creating-scripts.md)
- [Transform](components/transform.md)
- [Input](systems/input.md)
- [Object Model Notes](../architecture/object-model.md)
- [Registration And Archetypes](advanced/registration-and-archetypes.md)

## Components and Systems

- [SpriteRenderer](components/sprite-renderer.md)
- [SpriteAnimator](components/sprite-animator.md)
- [Sorting](components/sorting.md)
- [CursorInteractable](components/cursor-interactable.md)
- [Collider2D / PhysicsCollision](components/physics-collision.md)
- [Audio](systems/audio.md)
- [Tilemap](tilemap/tilemap.md)

## Shared Pattern

```rust
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine, RunaArchetype,
};
use runa_engine::runa_core::{
    ocs::{Object, Script, ScriptContext, World},
    components::SpriteRenderer,
};

struct MyBehavior;

impl Script for MyBehavior {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        let _ = (ctx, dt);
    }
}

#[derive(RunaArchetype)]
#[runa(name = "player")]
struct PlayerArchetype;

impl PlayerArchetype {
    fn create(world: &mut World) -> u64 {
        world.spawn(
            Object::new("Player")
                .with(SpriteRenderer::default())
                .with(MyBehavior)
        )
    }
}

fn register_game_types(engine: &mut Engine) {
    engine.register_script::<MyBehavior>();
    engine.register_archetype::<PlayerArchetype>();
}

fn main() {
    let mut engine = Engine::new();
    register_game_types(&mut engine);

    let world_rc = engine.create_world();
    world_rc.borrow_mut().spawn_archetype::<PlayerArchetype>();

    let _ = RunaApp::run_with_config(
        world_rc,
        RunaWindowConfig {
            title: "Tutorial".to_string(),
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: true,
            show_fps_in_title: true,
            window_icon: None,
        },
    );
}
```

## Why The Tutorials Use This Style

This style keeps:

- composition explicit
- bootstrap explicit
- behavior local to scripts
- editor dependency out of runtime code

It also gives Runa a future path for editor tools, archetype browsers, and serialization without moving the source of truth away from the runtime object model.

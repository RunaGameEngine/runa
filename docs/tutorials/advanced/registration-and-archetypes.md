# Registration And Archetypes

This guide covers the explicit bootstrap layer used for:

- component registration
- script registration
- archetype registration

It is useful for future editor/tooling integration, but it remains optional from the runtime point of view.

## Why Registration Exists

Runa keeps runtime construction code-first, but some systems still need stable type metadata:

- future generic serialization
- add-component / add-script tooling
- archetype browsers
- editor integration

The runtime registry stores:

- type name
- `TypeId`
- kind (`Component` or `Script`)
- built-in vs user registration origin
- optional runtime factory information for editor/tooling creation
- typed archetype factories keyed by `ArchetypeKey`

## Engine Bootstrap

```rust
use runa_engine::{Engine, RunaArchetype};
use runa_engine::runa_core::ocs::{Object, World};

#[derive(RunaArchetype)]
#[runa(name = "player")]
struct PlayerArchetype;

impl PlayerArchetype {
    fn create(world: &mut World) -> u64 {
        world.spawn(Object::new("Player"))
    }
}

fn register_game_types(engine: &mut Engine) {
    engine.register::<PlayerController>();
    engine.register_archetype::<PlayerArchetype>();
}
```

## Using Derives

You can attach metadata helpers to your own types:

```rust
use runa_engine::{Engine, RunaComponent, RunaScript};

#[derive(RunaComponent)]
pub struct Health {
    pub current: i32,
}

#[derive(RunaScript)]
pub struct PlayerController;

fn register_game_types(engine: &mut Engine) {
    Health::register(engine);
    PlayerController::register(engine);
}
```

The derive does not auto-register anything globally. Registration still happens explicitly in your bootstrap code.

For editor/tooling-visible fields, `RunaComponent` and `RunaScript` also expose:

- public fields
- private fields marked with `#[serialize_field]`

## Archetypes

Archetypes are reusable typed object factories:

```rust
use runa_engine::{Engine, RunaArchetype};
use runa_engine::runa_core::ocs::{Object, World};

#[derive(RunaArchetype)]
#[runa(name = "player")]
struct PlayerArchetype;

impl PlayerArchetype {
    fn create(world: &mut World) -> u64 {
        world.spawn(
            Object::new("Player")
                .with(PlayerController)
        )
    }
}

fn register_game_types(engine: &mut Engine) {
    engine.register_archetype::<PlayerArchetype>();
}
```

Spawn them later through the world:

```rust
let mut engine = Engine::new();
register_game_types(&mut engine);

let world_rc = engine.create_world();
world_rc.borrow_mut().spawn_archetype::<PlayerArchetype>();
```

String lookup still exists as a secondary API for tooling, serialization, and editor flows:

```rust
let _ = world.spawn_archetype_by_name("player");
```

## When To Use This

Use bootstrap registration when you want:

- one place to describe game runtime types
- reusable archetypes
- a future-safe path for editor/tooling

You do not have to use archetypes for every object. Manual `world.spawn(Object::new(...))` is still valid.

## Current Limits

- runtime factories are still required for editor-side creation of live types
- archetypes are typed code templates, not serialized prefab assets
- prefab/template unification is still incomplete

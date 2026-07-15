<!--
?? DEPRECATED � ECS Migration in Progress

This documentation refers to the old OCS (runa_core::ocs) system.
The engine is migrating to a new archetype-based ECS (runa_ecs crate).

See ROADMAP.md for the current migration track.
-->
# Why Registration Was Removed

Older versions of Runa required explicit registration of components, scripts, and archetypes through the engine's bootstrap layer. This file explains what registration was, why it existed, and why it is no longer needed.

## What Registration Used To Look Like

Previously, you had to:

1. Derive `RunaComponent` or `RunaScript` instead of `Component`
2. Derive `RunaArchetype` for reusable object factories
3. Call `engine.register::<T>()`, `engine.register_archetype::<T>()`, etc.
4. Use `world.spawn_archetype::<T>()` to spawn registered archetypes

```rust
// OLD approach — no longer valid
use runa_engine::{Engine, RunaArchetype, RunaComponent, RunaScript};

#[derive(RunaComponent)]
struct Health { current: i32 }

#[derive(RunaScript)]
struct PlayerController;

#[derive(RunaArchetype)]
struct PlayerArchetype;

fn main() {
    let mut engine = Engine::new();
    engine.register::<Health>();
    engine.register::<PlayerController>();
    engine.register_archetype::<PlayerArchetype>();

    let world_rc = engine.create_world();
    world_rc.borrow_mut().spawn_archetype::<PlayerArchetype>();
}
```

## Why Registration Existed

Registration was introduced to support:

- **Editor/tooling metadata** — the engine needed a way to discover all component and script types
- **Serialization bootstrap** — generic save/load needed type information at runtime
- **Archetype factories** — reusable named object templates that could be spawned by name

In theory, this kept a stable catalog of types that the editor and serialization layer could introspect without relying on reflection.

## Why It Was Removed

In practice, registration introduced friction without delivering enough value at the current stage:

- Every new component needed a separate registration call, making it easy to forget
- The `RunaArchetype`/`RunaComponent`/`RunaScript` derives were additional derives beyond `Component`
- The engine bootstrap layer (`Engine::new()` → `register` → `create_world`) added ceremony to the startup path
- Archetype registration (`spawn_archetype::<T>()`) duplicated what simple factory functions already provided
- Editor and serialization metadata were not yet using the registries, so the complexity was premature

## The Code-First Replacement

With the code-first API, registration is completely eliminated:

```rust
// NEW approach
use runa_engine::prelude::*;

#[derive(Component)]
struct Health { current: i32 }

#[derive(Component)]
struct PlayerController;

fn main() {
    let mut world = World::new();
    world.spawn_bundle((
        Transform::default(),
        SpriteRenderer::new(None),
        PlayerController,
    ));
}
```

- Derive `#[derive(Component)]` on any struct — no registration
- `world.spawn_bundle((...))` creates an object with the given components
- `world.spawn_object(Object::new("name").with(...))` for named objects
- Use plain Rust functions instead of archetypes for reusable object factories
- `world.find_all_with::<T>()` replaces `world.query::<T>()`

## Reusable Object Factories (Alternatives to Archetypes)

Instead of archetypes, use plain functions:

```rust
fn spawn_player(world: &mut World) -> ObjectId {
    world.spawn_bundle((
        Transform::default(),
        SpriteRenderer::new(Some(load_image!("assets/player.png"))),
        Health { current: 100, max: 100 },
        PlayerController,
    ))
}

fn spawn_enemy(world: &mut World, position: Vec3) -> ObjectId {
    let id = world.spawn_bundle((
        Transform::from_position(position),
        SpriteRenderer::new(Some(load_image!("assets/enemy.png"))),
        Health { current: 50, max: 50 },
        EnemyAi,
    ));
    id
}
```

## For Editor and Tooling Users

> **Editor is frozen.** The `#[serialize_field]` attribute and the registration
> system described on this page are archived. When editor work resumes, it will
> use opt-in metadata annotations rather than a mandatory registration system.

## Summary

| Old System | New Code-First API |
|---|---|
| `#[derive(RunaComponent)]` | `#[derive(Component)]` |
| `#[derive(RunaScript)]` | `#[derive(Component)]` + `impl Script` |
| `#[derive(RunaArchetype)]` | Plain Rust functions |
| `engine.register::<T>()` | Not needed |
| `engine.register_archetype::<T>()` | Not needed |
| `world.spawn_archetype::<T>()` | `world.spawn_bundle((...))` or `world.spawn_object(...)` |
| `world.query::<T>()` | `world.find_all_with::<T>()` |
| `Engine::new()` → register → create_world | `World::new()` directly |


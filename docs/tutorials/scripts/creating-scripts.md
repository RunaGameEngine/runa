# Creating Scripts

Scripts are attachable behavior components in Runa.

They no longer construct objects. Composition is explicit and happens before the object enters the world.

## Current Script Model

A script:

- attaches to an existing `Object`
- reads and mutates that object's components
- queries the world through `ScriptContext`
- queues world mutations through commands

Current lifecycle:

1. `start()`
2. `update()`
3. `late_update()`

## Basic Script

```rust
use runa_engine::runa_core::ocs::{Script, ScriptContext};

pub struct MoveRight {
    speed: f32,
}

impl MoveRight {
    pub fn new() -> Self {
        Self { speed: 1.0 }
    }
}

impl Script for MoveRight {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position.x += self.speed * dt;
        }
    }
}
```

## Late Update

Use `late_update()` for behavior that must read the final state after all normal script updates in the current tick.

Typical examples:

- camera follow
- look-at helpers
- post-movement attachments

```rust
impl Script for CameraFollow {
    fn late_update(&mut self, ctx: &mut ScriptContext, _dt: f32) {
        let Some(target_id) = ctx.find_first_with::<PlayerTag>() else {
            return;
        };
        let Some(target_position) = ctx
            .get_object(target_id)
            .and_then(|object| object.get_component::<Transform>())
            .map(|transform| transform.position)
        else {
            return;
        };

        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position = target_position;
        }
    }
}
```

## Attaching A Script

```rust
fn create_object() -> Object {
    Object::new("Mover")
        .with(SpriteRenderer::default())
        .with(MoveRight::new())
}
```

## Querying The World

```rust
impl Script for EnemyAI {
    fn update(&mut self, ctx: &mut ScriptContext, _dt: f32) {
        if let Some(player_id) = ctx.find_first_with::<PlayerMarker>() {
            let player = ctx.get_object(player_id);
            let _ = player;
        }
    }
}
```

## Queuing World Changes

Do not rely on direct mutable world access from script update logic.

Use commands instead:

```rust
impl Script for Lifetime {
    fn update(&mut self, ctx: &mut ScriptContext, _dt: f32) {
        if let Some(id) = ctx.id() {
            ctx.commands().despawn(id);
        }
    }
}
```

The command is applied after the lifecycle/update phase.

## Why `construct()` Was Removed

Older Runa scripts used `construct()` to add components from inside the script.

That made scripts:

- behavior code
- object factories
- tooling integration points

at the same time.

The current model is simpler:

- object factories build the object
- scripts define runtime behavior
- world/object/component serialization can target the actual runtime graph

## Optional Derive

If you use the umbrella crate, you can mark script types with:

```rust
use runa_engine::RunaScript;

#[derive(RunaScript)]
pub struct EnemyAI;
```

That does not auto-register the script. It only gives you explicit bootstrap helpers like:

```rust
EnemyAI::register(&mut engine);
```

## Serialized Fields

The derive macros can expose editor/tooling-visible fields from scripts.

- public fields are serialized by default
- private fields can be exposed with `#[serialize_field]`

```rust
use runa_engine::RunaScript;

#[derive(RunaScript)]
pub struct EnemyAI {
    pub speed: f32,
    #[serialize_field]
    aggro_radius: f32,
    hidden_runtime_state: bool,
}
```

In this example:

- `speed` is exposed
- `aggro_radius` is exposed
- `hidden_runtime_state` stays runtime-only

In the editor, `Content Browser -> Live Rust -> New Rust Script` now generates a pure script file.
Archetypes are created separately through `New Rust Archetype`, which keeps behavior code and object/template code split cleanly.

## Migration Summary

- move object assembly out of scripts
- build objects explicitly with `Object::new(...).with(...)`
- attach behavior with `.with(MyScript::new())`
- use `ctx.commands()` for world mutations
- use `ctx.world()` / `ctx.find_first_with::<T>()` for simple queries
- use `late_update()` for follow/attachment behavior that depends on final per-tick positions

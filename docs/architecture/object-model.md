# Object Model

This note describes the current Runa runtime architecture after the object-first refactor.

## Runtime Contract

Runa now treats the runtime object graph as the primary model:

- `World`
- `Object`
- components
- behavior scripts
- runtime registry metadata
- optional archetype catalog

The editor is expected to adapt to this model later. The runtime does not depend on editor-only construction rules.

## Object

`Object` is the entity/container unit in the world.

It contains:

- `ObjectId`
- a name
- a component container

`Transform` is a normal component, but it is auto-added in `Object::new(...)` and guaranteed to exist. There is no embedded-transform dual model anymore.

## Components

Components are the unit of runtime state and attachment.

Examples:

- `Transform`
- `SpriteRenderer`
- `MeshRenderer`
- `Camera`
- `AudioSource`
- `Collider2D`
- marker components
- script components

Runa intentionally avoids string-based tag systems as the primary gameplay linking mechanism. Prefer typed marker/data components instead.

## Scripts

Scripts are attachable behavior components.

They do not build objects anymore.

Lifecycle today:

1. `start()`
2. `update()`

Access happens through `ScriptContext`:

- own object id / handle
- own components
- read-only world queries
- deferred commands

That means script code looks like behavior code, not factory code.

## World Access

`World` no longer exposes its internal object storage directly.

The intended public surface is:

- `spawn(...)`
- `despawn(id)`
- `object(id)` / `object_mut(id)` — get the whole Object
- `get::<T>(id)` / `get_mut::<T>(id)` — get a specific component
- `find_first_with::<T>()`
- `find_all_with::<T>()`
- `query::<T>()`

Queries return `ObjectId`s. Order is intentionally not part of the contract.

## Deferred Commands

World mutations during script lifecycle are deferred.

Current command path:

```rust
if let Some(id) = ctx.id() {
    ctx.commands().despawn(id);
}
```

Commands are applied after lifecycle/update phases. This avoids invalidating iteration while the world is running behavior.

## Registration

Runa now has a runtime-owned metadata registry:

- component registration
- script registration
- archetype registration

This registry is currently metadata-only:

- type name
- `TypeId`
- kind

That is enough to establish stable bootstrap points for future serialization/editor work without forcing reflection-heavy runtime behavior.

## Archetypes

Archetypes are code-first reusable object factories.

Example:

```rust
engine.register_archetype::<PlayerArchetype>();

let mut world = engine.create_world();
let _ = world.spawn_archetype::<PlayerArchetype>();
```

This gives Runa a prefab-like reuse mechanism without moving composition into editor data or hidden serialized references.

## Why `construct()` Was Removed

Older Runa scripts used `construct()` to add components.

That made scripts:

- behavior
- object factories
- tooling bridge points

all at once.

That coupling became a bottleneck.

Now:

- composition happens in object factories / archetypes / bootstrap code
- scripts only describe runtime behavior
- serialization can target object/component state
- editor compatibility can be built on the same runtime model

## Current Limits

The new core model is much cleaner, but not everything is finished:

- generic component serialization is not registry-driven yet
- one concrete component type per object is still the storage rule
- `Object` still internally uses a raw world pointer
- fixed update / events are not implemented yet

So the architectural direction is in place, but some lower-level runtime cleanup is still ahead.

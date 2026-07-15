<!--
⚠️ ROADMAP IS BEING REWRITTEN — ECS MIGRATION IN PROGRESS

The old OCS (Object/Component/System) has been removed.
The engine now uses `runa_ecs` exclusively.
-->

# Runa Engine Roadmap — ECS Migration Track

> **Active migration (v0.6+):** `runa_core::ocs` → `runa_ecs` crate.
> Old OCS documentation is outdated and has been removed.

## Immediate (current session)

- [x] `runa_ecs` crate: BlobVec, Archetype, World, Query (Fetch GAT), Bundle macro
- [x] `#[system]` proc macro with inventory-based auto-registration
- [x] Scheduler integrated into `App` — auto-runs in fixed-timestep loop
- [x] `runa_engine` re-exports `runa_ecs`; `runa_app` hosts ECS World + Scheduler

## Next

- [ ] Port Script-based logic to `#[system]` functions
- [ ] Move component defs from `runa_core::components` to plain structs
- [ ] Add command queue (deferred spawn/despawn) to `runa_ecs::World`
- [ ] Remove `runa_core::ocs` and `runa_core::codefirst`

## Performance invariants (new ECS)

These must never regress:
- Unused features must have zero runtime cost (Cargo features + `#[cfg]`)
- No per-frame GPU buffer allocations — use pools or persistent buffers
- Archetype queries must not allocate per call
- Archetype column access is O(1) (index by ComponentInfo index)
- `Entity` lookup is O(1) via location map

## How to contribute

See [`CONTRIBUTING.md`](CONTRIBUTING.md) (docs being rewritten for ECS).

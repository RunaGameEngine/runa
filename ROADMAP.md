# Runa Engine Roadmap

> Philosophy: code-first runtime + convenient editor. Nothing that hurts performance,
> even when unused. Community-driven. Clarity and performance are the two pillars.

## Near-term (0.6.x — 0.7.x)

### Fix documentation drift
- [x] Fix `Camera::new_ortho(...)` → `Camera::new_orthographic(...)` in 2D tutorial
- [x] Fix `Camera::new_perspective(..., viewport_size)` signature in 3D tutorial (remove
      the stale 7th argument)
- [x] CHANGELOG mentions `new_ortho` — update to `new_orthographic`
- [ ] Document all currently undocumented public APIs (see list below)

### Performance — no-regressions baseline
- [ ] Per-frame GPU buffer allocation → pooled/ring-buffer strategy
      (`renderer.rs:1074-1108`, `renderer.rs:1165-1192`, `renderer.rs:1253-1268`)
- [ ] `world.object(id)` O(n) scan → `HashMap<ObjectId, usize>` O(1) lookup
- [ ] Remove-and-reinsert component pattern → shared borrows (`object.rs:246-282`)
- [ ] Sprite batching by texture key (currently one batch per sprite, `renderer.rs:969`)
- [ ] Add criterion benchmarks for hot paths (render queue building, queries, transforms)
- [ ] Add CI with `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check`

### Plugin system (trait-based, zero-cost)
- [ ] Define `RunaPlugin` trait with `fn build(&self, app: &mut AppBuilder)`
- [ ] Feature-gate everything non-essential behind Cargo features
      (`physics = ["dep:rapier"]`, `editor = ["dep:egui"]`, etc.)
- [ ] Publish core crates on crates.io so community plugins can depend on stable version

### Documentation overhaul
- [ ] Document all public APIs in code (rustdoc)
- [ ] Tutorials for: EventBus, hierarchy API, DebugRenderer, Console,
      ScriptContext full API, Runa3D import pipeline, CanvasSpace UI
- [ ] Add doc examples for every component type
- [ ] Architecture decision records (ADR) for major past decisions

## Medium-term (0.8.x — 0.9.x)

### 2D polish (first-class 2D experience)
- [ ] Pixel-perfect 2D pipeline (pixel snapping, sub-pixel rendering)
- [ ] 2D lighting (normal maps for sprites, 2D shadow casting)
- [ ] Particle system (GPU-based, 2D)
- [ ] Physics integration via Rapier (feature-gated, zero-cost when off)
- [ ] Tilemap batching — chunked/chunked instance buffers for large maps
- [ ] UI layout engine (measure/arrange, text wrapping, anchoring, stretch)
- [ ] Skeleton animation / spritesheet state machine

### Editor maturity
- [ ] Live asset hot-reload (textures, audio, scenes)
- [ ] Scene graph / hierarchy drag-drop from editor
- [ ] Inspector: multi-object editing, prefab overrides
- [ ] Play-In-Editor (embedded viewport instead of separate window)
- [ ] Gizmo improvements (scale, rotation arcball, grid snapping)
- [ ] Editor plugin API (community can extend editor panels)

### 3D foundations
- [ ] PBR material pipeline (metallic-roughness, normal maps, occlusion)
- [ ] Shadow maps (directional + point light)
- [ ] glTF import polish (materials, animations, skinning)
- [ ] Skybox / skysphere
- [ ] Fog (distance + height)

## Long-term (0.10.x +)

### 3D expansion
- [ ] Deferred or clustered rendering for many lights
- [ ] Skeletal animation (glTF skinning)
- [ ] Post-processing stack (bloom, tonemapping, DOF)
- [ ] HDR environment lighting
- [ ] Multi-window runtime support

### Community & ecosystem
- [ ] Official plugin registry / curated list
- [ ] Template-based project generator (`runa new my-game`)
- [ ] WASM build target support
- [ ] Mobile (Android/iOS) rendering path
- [ ] Runtime scripting (WASM or Lua sandbox, optional feature)

## Undocumented APIs (need docs before 0.7)

These exist in code but have zero or insufficient documentation.
Priority order:
1. `EventBus` + `ScriptContext::{emit_event, subscribe_to_event}` — primary
   inter-object communication
2. `InteractionSystem` — the system driving `CursorInteractable`
3. `Console` — built-in dev console (backquote key)
4. `Runa3D / .r3m` — glTF-to-native import pipeline
5. Object hierarchy (`set_parent()`, `root_object_ids()`, `is_descendant_of()`)
6. `ScriptContext` full API (`get_object()`, `find_first_with()`, `colliding_2d()`,
   `world()`, `emit_event()`, etc.)
7. `DebugRenderer` — debug collision visualization
8. `ObjectBuilder` — alternative object construction
9. `CanvasSpace::{Screen, Camera, World}` — UI coordinate spaces
10. `Transform::{rotate_x/y/z}`, interpolation fields
11. `Mesh::cube()`, `Mesh::quad()`, `Mesh::plane()`, `Mesh::pyramid()`
12. `Material` full PBR fields
13. `RunaTypeRegistration` trait
14. Project scaffold API (`create_empty_project()`, templates)
15. Fixed-timestep loop (60 FPS + interpolation)
16. `RunaApp::run_default()` — default config shortcut
17. All serialization asset types (`*Asset`)
18. `F11` fullscreen toggle (built-in)
19. `Camera` full API (`forward()`, `ortho_visible_size()`, `resize()`, etc.)

## Performance invariants

These must never regress:
- Unused features must have zero runtime cost (Cargo features + `#[cfg]`)
- No per-frame GPU buffer allocations — use pools or persistent buffers
- No O(n) scans in hot paths — `ObjectId` → `Object` must be O(1)
- No component remove-and-reinsert in lifecycle — use shared borrows
- Archetype queries must not allocate per call
- Scripts that do nothing should cost nothing (default trait methods)

## How to contribute

See [`CONTRIBUTING.md`](CONTRIBUTING.md).

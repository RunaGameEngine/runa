# Runtime And Editor Update Notes

This document summarizes the larger runtime/editor changes that landed in `0.5.1-alpha.1`.

## Runtime

- Added `late_update()` to the script lifecycle
- `World::update()` now performs:
  1. `update()` for all objects
  2. `late_update()` for all objects
- `Transform` interpolation is used consistently by the render path for 2D and 3D objects
- Camera constructors no longer expose `viewport_size`
- `viewport_size` is runtime-owned and updated from the active render target/window size

## Cameras

- Orthographic cameras now use a consistent right-handed view/projection path
- Orthographic visible size is aspect-aware through `Camera::ortho_visible_size()`
- `screen_to_world()` and editor overlay helpers use the same visible-size model
- Editor and runtime camera handling were aligned more closely to reduce fullscreen/wide-screen drift

## Registry And Serialization

- Typed archetypes remain the primary gameplay-facing template path
- Runtime registry metadata now carries richer information for editor/tooling usage
- Derived components and scripts can expose serialized fields through:
  - `#[serialize_field]` only
- Recommended editor-visible defaults now come from `Default`
  - if a type already uses `new()`, implement `Default` by delegating to `new()`
- Editor/project flows can store serialized script/component state even when the editor process does not own the concrete runtime type directly
- `SpriteRenderer` now carries `pixels_per_unit`, and the value is serialized through project/world assets

## Editor

- `runa_editor` was split into smaller modules to reduce the size of the old monolithic app file
- Inspector presentation was aligned around:
  - `Object`
  - `Transform`
  - `Components`
  - `Scripts`
- Component and script creation now consult the runtime type registry instead of hardcoded editor-side lists
- SVG icon loading was added for editor-only UI/icon assets
- `Project Settings` now exposes the manifest-backed app/window settings used by `Play In Window`
- `Build Settings` now covers build profile, output directory, and Windows release console-hiding
- `Content Browser -> Live Rust` now separates script-file and archetype-file creation instead of generating both in one template

## Serialized Runtime Data

- Project/world loading now reapplies serialized fields onto existing runtime script/component instances instead of dropping them when the runtime type is already present
- Archetype-backed object overrides keep serialized script/component entries when the editor reconstructs the object from project metadata

## Current Known Follow-Ups

- There is still no full pixel-perfect 2D pipeline
- Runtime pixel snapping is not implemented yet
- Some warnings remain in `runa_render` and `runa_editor`
- Prefab/template unification is still incomplete

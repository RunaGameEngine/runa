# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Runtime lifecycle**
  - Added `late_update()` to the script lifecycle
  - `World::update()` now runs `update()` for all objects first and `late_update()` in a second pass

- **Runtime/editor type metadata**
  - Added serialized field metadata for runtime-owned components and scripts
  - Added support for generic serialized component/script storage for editor-facing project data
  - Added project metadata snapshots for archetypes, components, and scripts

- **Editor asset support**
  - Added editor-only SVG icon loading with PNG fallback
  - Added component icon support in the viewport and inspector
  - Added `folder-empty` handling and root-folder navigation in the content browser

- **Editor workflow**
  - Added runtime-registry-driven object creation for:
    - empty objects
    - archetypes
    - components
    - scripts
  - Added inspector separation for `Object`, `Transform`, `Components`, and `Scripts`
  - Added bottom inspector actions for adding components and scripts
  - Added manifest-backed `Project Settings` for `RunaAppConfig` values used by `Play In Window`
  - Added `Build Settings` and `Build Game` flow for cargo-driven project builds
  - Added `Content Browser -> Live Rust` split between script-file and archetype-file creation

### Changed

- **Camera API**
  - `viewport_size` was removed from public camera constructors
  - `Camera::new_ortho(width, height)` now stores world-visible size directly
  - `viewport_size` is now runtime-owned state updated by the app/editor from the actual render target size

- **Orthographic camera behavior**
  - Orthographic projection now uses a consistent `look_at_rh` view path
  - Orthographic camera orientation was aligned to `-Z`
  - Orthographic visible area, editor overlays, and `screen_to_world()` now use the same aspect-aware visible size model

- **Editor architecture**
  - `runa_editor` was split into smaller `editor_app` modules for UI, viewport, project flow, world operations, helpers, and placeables
  - Inspector rows now follow a unified `label on the left / controls on the right` layout
  - Mesh renderer editing now supports selecting built-in meshes from the inspector

- **Examples and scaffolding**
- Project scaffolding now generates typed-archetype-first runtime bootstrap
- Project scaffolding now writes manifest-backed window config and Windows release subsystem setup
- Bundled examples were updated to the newer camera constructors and typed archetype flow
- `sandbox` camera follow now runs in `late_update()`

### Fixed

- Editor viewport color mismatch caused by incorrect sRGB handling when sampling offscreen targets through `egui`
- Orthographic camera rendering issues related to aspect handling, direction, and visible bounds
- `screen_to_world()` drift and cursor interaction issues at wide resolutions/fullscreen
- Runtime/editor mismatch when using orthographic cameras with resized viewports
- Editor/project metadata refresh now updates archetype/component/script lists from project bootstrap metadata
- Project scripts and serialized fields are now visible again in editor-backed inspection flows
- Serialized script/component overrides now reapply onto existing runtime instances during world loading
- Archetype-backed world objects now preserve serialized script/component override data through editor reconstruction
- Final game rendering now matches source sprite colors more closely than the old editor viewport path

## [0.3.0-alpha.1] - 2026-04-20

### Added

- **Typed archetype API**
  - Added `ArchetypeKey` and `RunaArchetype`
  - Added typed registration via `engine.register_archetype::<T>()`
  - Added typed spawning via `world.spawn_archetype::<T>()`
  - Added `#[derive(RunaArchetype)]` with deterministic snake_case keys
  - Added optional `#[runa(name = \"...\")]` override for archetype names

- **Automatic built-in type registration**
  - `Engine::new()` now registers built-in runtime components automatically
  - Added registry metadata source tracking for built-in vs user registrations
  - Added generic user-type registration via `engine.register::<T>()` for derived components and scripts

- **Runtime registry metadata**
  - Added built-in/user origin metadata for registered components, scripts, and archetypes
  - Exposed queries for built-in and user type sets to support tooling and editor work

- **Documentation and examples refresh**
  - Updated README and tutorials to use the typed archetype flow
  - Updated bundled examples to remove string-based archetype usage from gameplay code
  - Added tests covering typed archetypes and automatic built-in registration

### Changed

- **Breaking:** Archetype registration is now type-driven in normal gameplay code
  - `engine.register_archetype_named(...)` is no longer the primary API
  - `world.spawn_archetype_by_name(...)` remains as a secondary path for tooling, serialization, and editor integration

- **Breaking:** Engine built-in components no longer require user registration
  - User bootstrap code should register only game-specific components, scripts, and archetypes

- **Runtime architecture**
  - Continued the object-first OCS/runtime cleanup around `World`, `Object`, `ScriptContext`, deferred commands, and runtime-owned registry metadata
  - Kept editor/tooling-oriented string lookup as an internal/secondary mechanism instead of a gameplay-facing default

### Fixed

- Reduced fragile setup around engine bootstrap by removing required manual registration of core engine components
- Removed string-only archetype entry points from bundled gameplay examples
- Brought primary docs in line with the typed archetype and explicit registration model

## [0.2.0-alpha.2] - 2026-03-27

### Added

- **Unified Camera System**
  - New `Camera` component supporting both 2D orthographic and 3D perspective projections
  - `Camera::new_ortho()` - Simple 2D camera setup
  - `Camera::new_perspective()` - Full 3D camera with position, target, FOV
  - Automatic aspect ratio correction for proper rendering
  - `screen_to_world()` conversion for accurate mouse input

- **Documentation**
  - Complete 2D game creation guide
  - Complete 3D game creation guide with FPS controller
  - Updated tutorials README with camera system documentation
  - Quick start guides for both 2D and 3D development paths

- **Input System**
  - Proper camera integration for `get_mouse_world_position()`
  - Aspect ratio correction in screen-to-world conversion
  - Fixed cursor interaction with correct world coordinates

### Changed

- **Breaking:** Camera component refactored
  - `Camera2D` and `Camera3D` deprecated in favor of unified `Camera`
  - Old components remain for backward compatibility with deprecation warnings
  - Migration path: Replace `Camera2D::new()` with `Camera::new_ortho()`
  - Migration path: Replace `Camera3D { ... }` with `Camera::new_perspective()`

- **Rendering Pipeline**
  - Fixed depth-stencil attachment for sprite pipeline compatibility
  - Proper depth buffer handling for mixed 2D/3D scenes
  - Single render pass for all objects (no more multiple submit calls)
  - Improved performance with batched rendering

- **Interaction System**
  - `InteractionSystem` now updates before scripts for accurate hover state
  - Fixed `CursorInteractable` with proper camera integration
  - Mouse drag now works correctly with aspect ratio correction

### Fixed

- Black screen in 2D scenes (ortho_size now properly calculated)
- Mouse position offset in `CursorInteractable` (aspect ratio correction)
- Depth-stencil format mismatch in sprite rendering
- Camera viewport size not updating from active camera
- Input system using wrong camera for world position

### Deprecated

- `Camera2D` - Use `Camera::new_ortho()` instead
- `Camera3D` - Use `Camera::new_perspective()` instead

## [0.2.0-alpha.1] - 2026-03-26

### Added

- **3D Camera System**
  - `Camera3D` component with perspective projection
  - `ActiveCamera` marker component for explicit camera selection
  - Automatic camera fallback: ActiveCamera → First Camera3D → Warning
  - Safe rendering when no camera present (black screen, no crash)

- **Cursor Control API**
  - `input_system::show_cursor()` - Show/hide cursor
  - `input_system::lock_cursor()` - Lock/unlock cursor to window
  - `input_system::set_cursor_mode()` - Combined cursor control
  - Global access from anywhere in scripts

- **3D Sandbox Example**
  - `sandbox_3d` - First-person camera controller
  - WASD movement + Space/Ctrl vertical movement
  - Mouse look with locked cursor (right-click toggle)
  - Inverted Y-axis for FPS-style control

- **Input Improvements**
  - `get_mouse_delta()` now uses `DeviceEvent::MouseMotion`
  - Works correctly when cursor is locked
  - No more input lag or single-frame issues

### Changed

- **Breaking:** Camera system now requires explicit camera component
  - Removed automatic default camera creation
  - Add `Camera3D` or `Camera2D` component to enable rendering
  - Use `ActiveCamera` marker for explicit camera selection

- **Breaking:** `AudioSource::play()` API
  - Removed `world` parameter from `Script::update()`
  - Audio playback via `audio.play()` instead of `world.play_sound()`
  - `play_on_awake` flag for automatic playback

- Version bumped to 0.2.0-alpha.1 (3D rendering milestone)

### Documentation

- Updated README.md with 3D camera examples
- Added ActiveCamera usage guide
- Updated cursor control documentation
- Added troubleshooting for "No camera found" warning

## [0.1.3-alpha.1] - 2026-03-26

### Added

- **3D Spatial Audio System**
  - `AudioListener` component for camera/player
  - Distance-based volume attenuation
  - Stereo panning simulation
  - `stereo_separation` parameter

- **AudioSource Improvements**
  - `play_on_awake` flag
  - `play()` and `stop()` methods
  - `min_distance` and `max_distance` controls

- **sandbox_soundtest** example for audio testing

### Changed

- `Script::update()` signature simplified (removed `world` parameter)
- Audio playback via `AudioSource::play()` component method

## [Unreleased] %% 0.1.0 %%

### Added

- Initial project structure with workspace setup
- Core OCS system (`World`, `Object`, components)
- `Transform` component (mandatory for all objects)
- `Script` trait with lifecycle methods (`construct`, `start`, `update`)
- Global `Input` API for keyboard/mouse access anywhere in code
- 2D rendering pipeline with sprite batching (1000+ objects support)
- Tilemap component with negative coordinate support and texture batching
- `CursorInteractable` component for mouse interaction with objects
- Basic audio system using `rodio` (play/stop sounds)
- Experimental 3D mesh pipeline with depth buffer and instancing
- Camera2D with aspect ratio correction and screen-to-world conversion
- Fullscreen toggle (F11)

### Fixed

- Vertex buffer overwriting causing texture flickering in tilemaps
- Mouse world position calculation with aspect ratio correction
- Bind group caching for 10x rendering performance boost
- Z-fighting prevention in 3D pipeline (proper near/far planes)

### Changed

- Removed `input()` method from `Script` trait (replaced with global `Input` API)
- Unified texture handling: `Arc<TextureAsset>` instead of `Handle`
- Renderer now uses single vertex buffer with offsets for all draw calls
- Camera matrix calculation inverted Y for proper screen coordinates

### Deprecated

- None

### Removed

- None

### Security

- None

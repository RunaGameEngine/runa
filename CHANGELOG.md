# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
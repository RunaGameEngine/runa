# Creating a 3D Game

This guide shows the current recommended Runa pattern for a small 3D scene.

## Main File

```rust
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine, RunaArchetype,
};
use runa_engine::runa_core::ocs::World;

mod camera_controller;
mod rotating_cube;

#[derive(RunaArchetype)]
#[runa(name = "camera-controller")]
struct CameraControllerArchetype;

impl CameraControllerArchetype {
    fn create(world: &mut World) -> u64 {
        world.spawn(camera_controller::create_camera_controller())
    }
}

#[derive(RunaArchetype)]
#[runa(name = "rotating-cube")]
struct RotatingCubeArchetype;

impl RotatingCubeArchetype {
    fn create(world: &mut World) -> u64 {
        world.spawn(rotating_cube::create_rotating_cube())
    }
}

fn main() {
    let mut engine = Engine::new();
    engine.register_archetype::<CameraControllerArchetype>();
    engine.register_archetype::<RotatingCubeArchetype>();

    let mut world = engine.create_world();
    let _ = world.spawn_archetype::<CameraControllerArchetype>();
    let _ = world.spawn_archetype::<RotatingCubeArchetype>();

    let config = RunaWindowConfig {
        title: "My 3D Game".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(world, config);
}
```

## Camera Controller

```rust
use runa_engine::runa_core::{
    components::{ActiveCamera, Camera},
    glam::Vec3,
    ocs::{Object, Script, ScriptContext},
};

pub struct CameraController {
    position: Vec3,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
        }
    }
}

impl Script for CameraController {
    fn update(&mut self, ctx: &mut ScriptContext, _dt: f32) {
        if let Some(camera) = ctx.get_component_mut::<Camera>() {
            camera.position = self.position;
        }
    }
}

pub fn create_camera_controller() -> Object {
    Object::new("Main Camera")
        .with(Camera::new_perspective(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 6.0),
            Vec3::Y,
            75.0_f32.to_radians(),
            0.1,
            1000.0,
        ))
        .with(ActiveCamera)
        .with(CameraController::new())
}
```

## Rotating Cube

```rust
use runa_engine::runa_core::{
    components::{Mesh, MeshRenderer, Transform},
    glam::{Quat, Vec3},
    ocs::{Object, Script, ScriptContext},
};

pub struct RotatingCube {
    rotation_speed: f32,
}

impl RotatingCube {
    pub fn new() -> Self {
        Self { rotation_speed: 0.5 }
    }
}

impl Script for RotatingCube {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.rotation *= Quat::from_rotation_y(self.rotation_speed * dt);
        }
    }
}

pub fn create_rotating_cube() -> Object {
    Object::new("Cube")
        .with(Transform {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            previous_position: Vec3::ZERO,
            previous_rotation: Quat::IDENTITY,
        })
        .with(MeshRenderer::new(Mesh::cube(1.0)))
        .with(RotatingCube::new())
}
```

## Why Composition Is Explicit

The camera and cube are still just objects with components. Typed archetypes only make those factories reusable.

That separation keeps:

- rendering/camera data visible at creation time
- script behavior focused on runtime logic
- future editor integration pointed at the same runtime object model

## Next Steps

- [Creating Scripts](../scripts/creating-scripts.md)
- [Input](../systems/input.md)
- [Renderer Notes](../../architecture/renderer.md)

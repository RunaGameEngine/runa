# Creating a 3D Game

This guide shows the current recommended Runa pattern for a small 3D scene.

## Main File

```rust
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine,
};
use runa_engine::runa_core::ocs::World;

mod camera_controller;
mod rotating_cube;


fn main() {
    let engine = Engine::new();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.bottow_mut();

        world.spawn_object(create_camera_controller());
        world.spawn_object(create_rotating_cube());
    }

    let config = RunaWindowConfig {
        title: "My 3D Game".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(world_rc, config);
}
```

## Camera Controller

```rust
use runa_core::components::{ActiveCamera, Camera};
use runa_core::glam::Vec3;
use runa_core::input;
use runa_core::input::get_mouse_delta;
use runa_core::input::InputState;
use runa_core::ocs::{Object, Script, ScriptContext};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

static mut CURSOR_LOCKED: bool = false;

pub struct CameraController {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: 3.0,
            sensitivity: 0.001,
        }
    }

    fn get_forward(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
        .normalize()
    }

    fn get_right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()).normalize()
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}

impl Script for CameraController {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if InputState::is_mouse_button_just_pressed(MouseButton::Right) {
            unsafe {
                CURSOR_LOCKED = !CURSOR_LOCKED;
                input::show_cursor(!CURSOR_LOCKED);
                input::lock_cursor(CURSOR_LOCKED);
            }
        }

        unsafe {
            if CURSOR_LOCKED {
                let mouse_delta = get_mouse_delta();
                self.yaw -= mouse_delta.0 * self.sensitivity;
                self.pitch -= mouse_delta.1 * self.sensitivity;
                self.pitch = self.pitch.clamp(-1.5, 1.5);
            }
        }

        let forward = self.get_forward();
        let right = self.get_right();
        let mut movement = Vec3::ZERO;

        if InputState::is_key_pressed(KeyCode::KeyW) {
            movement += forward;
        }
        if InputState::is_key_pressed(KeyCode::KeyS) {
            movement -= forward;
        }
        if InputState::is_key_pressed(KeyCode::KeyD) {
            movement += right;
        }
        if InputState::is_key_pressed(KeyCode::KeyA) {
            movement -= right;
        }
        if InputState::is_key_pressed(KeyCode::Space) {
            movement += Vec3::Y;
        }
        if InputState::is_key_pressed(KeyCode::ControlLeft)
            || InputState::is_key_pressed(KeyCode::ControlRight)
        {
            movement -= Vec3::Y;
        }

        if movement.length() > 0.0 {
            self.position += movement.normalize() * self.speed * dt;
        }

        let target = self.position
            + Vec3::new(
                -self.yaw.sin() * self.pitch.cos(),
                self.pitch.sin(),
                -self.yaw.cos() * self.pitch.cos(),
            );

        if let Some(camera) = ctx.get_component_mut::<Camera>() {
            camera.position = self.position;
            camera.target = target;
            camera.up = Vec3::Y;
        }
    }
}

pub fn create_camera_controller() -> Object {
    Object::new("Main Camera")
        .with(Camera::new_perspective(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 6.0),
            Vec3::Y,
            75.0,
            0.1,
            1000.0,
        ))
        .with(ActiveCamera)
        .with(CameraController::new())
}

```

## Camera Controller

```rust
use runa_core::components::{ActiveCamera, Camera};
use runa_core::glam::Vec3;
use runa_core::input;
use runa_core::input::get_mouse_delta;
use runa_core::input::InputState;
use runa_core::ocs::{Object, Script, ScriptContext};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

static mut CURSOR_LOCKED: bool = false;

pub struct CameraController {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    fn get_forward(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
        .normalize()
    }

    fn get_right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()).normalize()
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: 3.0,
            sensitivity: 0.001,
        }
    }
}

impl Script for CameraController {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if InputState::is_mouse_button_just_pressed(MouseButton::Right) {
            unsafe {
                CURSOR_LOCKED = !CURSOR_LOCKED;
                input::show_cursor(!CURSOR_LOCKED);
                input::lock_cursor(CURSOR_LOCKED);
            }
        }

        unsafe {
            if CURSOR_LOCKED {
                let mouse_delta = get_mouse_delta();
                self.yaw -= mouse_delta.0 * self.sensitivity;
                self.pitch -= mouse_delta.1 * self.sensitivity;
                self.pitch = self.pitch.clamp(-1.5, 1.5);
            }
        }

        let forward = self.get_forward();
        let right = self.get_right();
        let mut movement = Vec3::ZERO;

        if InputState::is_key_pressed(KeyCode::KeyW) {
            movement += forward;
        }
        if InputState::is_key_pressed(KeyCode::KeyS) {
            movement -= forward;
        }
        if InputState::is_key_pressed(KeyCode::KeyD) {
            movement += right;
        }
        if InputState::is_key_pressed(KeyCode::KeyA) {
            movement -= right;
        }
        if InputState::is_key_pressed(KeyCode::Space) {
            movement += Vec3::Y;
        }
        if InputState::is_key_pressed(KeyCode::ControlLeft)
            || InputState::is_key_pressed(KeyCode::ControlRight)
        {
            movement -= Vec3::Y;
        }

        if movement.length() > 0.0 {
            self.position += movement.normalize() * self.speed * dt;
        }

        let target = self.position
            + Vec3::new(
                -self.yaw.sin() * self.pitch.cos(),
                self.pitch.sin(),
                -self.yaw.cos() * self.pitch.cos(),
            );

        if let Some(camera) = ctx.get_component_mut::<Camera>() {
            camera.position = self.position;
            camera.target = target;
            camera.up = Vec3::Y;
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

## Camera Controller

```rust
use runa_core::components::{Mesh, MeshRenderer, Transform};
use runa_core::glam::{Quat, Vec3};
use runa_core::ocs::{Object, Script, ScriptContext};

pub struct RotatingCube {
    rotation_speed: f32,
}

impl Default for RotatingCube {
    fn default() -> Self {
        Self {
            rotation_speed: 0.5,
        }
    }
}

impl Script for RotatingCube {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            let rotation = Quat::from_rotation_y(self.rotation_speed * dt);
            transform.rotation *= rotation;
        }
    }
}

pub fn create_rotating_cube() -> Object {
    Object::new("Rotating Cube")
        .with(Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(1.0, 1.0, 1.0),
            previous_position: Vec3::ZERO,
            previous_rotation: Quat::IDENTITY,
        })
        .with(MeshRenderer::new(Mesh::cube(1.0)))
        .with(RotatingCube::default())
}

```

## Why Composition Is Explicit

The camera and the cube are still just objects with components.

You can also run `cargo run -p sandbox_3d` or check out the `examples/sandbox_3d` source code to see it in action.

That separation keeps:

- rendering/camera data visible at creation time
- script behavior focused on runtime logic
- future editor integration pointed at the same runtime object model

## Next Steps

- [Creating Scripts](../scripts/creating-scripts.md)
- [Input](../systems/input.md)
- [Renderer Notes](../../architecture/renderer.md)

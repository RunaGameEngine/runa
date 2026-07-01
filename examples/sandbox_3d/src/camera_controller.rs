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
            75.0_f32.to_radians(),
            0.1,
            1000.0,
        ))
        .with(ActiveCamera)
        .with(CameraController::new())
}

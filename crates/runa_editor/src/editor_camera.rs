use std::collections::HashSet;
use std::sync::Arc;

use runa_core::components::Camera;
use runa_core::glam::Vec3;
use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window};

pub struct EditorCameraController {
    camera: Option<Camera>,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
    fov: f32,
    sensitivity: f32,
    look_active: bool,
    viewport_hovered: bool,
    pressed_keys: HashSet<KeyCode>,
}

impl EditorCameraController {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 1.4, 6.0),
            yaw: 0.0,
            pitch: -0.24,
            speed: 4.0,
            sensitivity: 1.,
            look_active: false,
            viewport_hovered: false,
            pressed_keys: HashSet::new(),
            camera: None,
            fov: 75.0_f32,
        }
    }

    pub fn set_viewport_hovered(&mut self, hovered: bool) {
        self.viewport_hovered = hovered;
    }

    pub fn handle_window_event(&mut self, window: &Arc<Window>, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => self.handle_keyboard_input(event),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => {
                let should_activate = *state == ElementState::Pressed && self.viewport_hovered;
                self.set_look_active(window, should_activate);
                true
            }
            _ => false,
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if !self.look_active {
            return;
        }

        if let DeviceEvent::MouseMotion { delta } = event {
            self.yaw -= delta.0 as f32 * self.sensitivity / 100.;
            self.pitch -= delta.1 as f32 * self.sensitivity / 100.;
            self.pitch = self.pitch.clamp(-1.5, 1.5);
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut movement = Vec3::ZERO;
        let forward = self.forward();
        let right = self.right();

        if self.pressed_keys.contains(&KeyCode::KeyW) {
            movement += forward;
        }
        if self.pressed_keys.contains(&KeyCode::KeyS) {
            movement -= forward;
        }
        if self.pressed_keys.contains(&KeyCode::KeyD) {
            movement += right;
        }
        if self.pressed_keys.contains(&KeyCode::KeyA) {
            movement -= right;
        }
        if self.pressed_keys.contains(&KeyCode::Space) {
            movement += Vec3::Y;
        }
        if self.pressed_keys.contains(&KeyCode::ControlLeft) {
            movement -= Vec3::Y;
        }

        if movement.length_squared() > 0.0 {
            let speed = if self.pressed_keys.contains(&KeyCode::ShiftLeft)
                || self.pressed_keys.contains(&KeyCode::ShiftRight)
            {
                self.speed * 2.
            } else {
                self.speed
            };
            self.position += movement.normalize() * speed * dt;
        }
    }

    pub fn camera(&mut self, viewport_size: (u32, u32)) -> Camera {
        self.camera = Some(Camera::new_perspective(
            self.position,
            self.position + self.forward(),
            Vec3::Y,
            self.fov.to_radians(),
            0.1,
            1000.0,
            viewport_size,
        ));
        self.camera.unwrap()
    }

    pub fn get_speed(&mut self) -> f32 {
        self.speed
    }

    pub fn set_speed(&mut self, new_speed: f32) {
        self.speed = new_speed;
    }

    pub fn get_sensitivity(&mut self) -> f32 {
        self.sensitivity
    }

    pub fn set_sensitivity(&mut self, new_sens: f32) {
        self.sensitivity = new_sens;
    }

    /// Return radians
    pub fn get_fov(&mut self) -> f32 {
        self.fov
    }

    /// In radians
    pub fn set_fov(&mut self, new_fov: f32) {
        self.fov = new_fov;
    }

    pub fn shutdown(&mut self, window: &Arc<Window>) {
        self.set_look_active(window, false);
    }

    pub fn is_look_active(&self) -> bool {
        self.look_active
    }

    fn handle_keyboard_input(&mut self, event: &KeyEvent) -> bool {
        let PhysicalKey::Code(code) = event.physical_key else {
            return false;
        };

        if !self.look_active {
            return false;
        }

        match event.state {
            ElementState::Pressed => {
                self.pressed_keys.insert(code);
            }
            ElementState::Released => {
                self.pressed_keys.remove(&code);
            }
        }
        self.look_active
    }

    fn set_look_active(&mut self, window: &Arc<Window>, active: bool) {
        self.look_active = active;
        if !active {
            self.pressed_keys.clear();
        }
        window.set_cursor_visible(!active);
        let _ = window.set_cursor_grab(if active {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        });
    }

    fn forward(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
        .normalize()
    }

    fn right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()).normalize()
    }
}

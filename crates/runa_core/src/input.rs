use glam::Vec2;
use std::collections::HashSet;
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::components::camera2d::Camera2D;

#[derive(Default, Clone)]
pub struct InputState {
    pub keys_pressed: HashSet<KeyCode>,
    pub keys_just_pressed: HashSet<KeyCode>,
    // pub keys_just_released: HashSet<KeyCode>,
    pub mouse_position: (f32, f32),
    pub mouse_buttons_pressed: HashSet<MouseButton>,
    pub mouse_buttons_just_pressed: HashSet<MouseButton>,
    // pub mouse_buttons_just_released: HashSet<MouseButton>,
    pub mouse_wheel_delta: f32,

    pub camera: Option<Camera2D>,
}

impl InputState {
    pub fn default() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            // keys_just_released: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_just_pressed: HashSet::new(),
            // mouse_buttons_just_released: HashSet::new(),
            mouse_wheel_delta: 0.0,
            camera: None,
        }
    }

    pub fn update_frame(&mut self) {
        self.keys_just_pressed.clear();
        // self.keys_just_released.clear();
        self.mouse_buttons_just_pressed.clear();
        // self.mouse_buttons_just_released.clear();
        self.mouse_wheel_delta = 0.0;
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_pressed.contains(&button)
    }

    // pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
    //     self.mouse_buttons_just_released.contains(&button)
    // }

    pub fn get_mouse_world_position(&self) -> Option<Vec2> {
        if let Some(camera) = &self.camera {
            Some(camera.screen_to_world(self.mouse_position))
        } else {
            None
        }
    }
}

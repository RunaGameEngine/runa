use glam::Vec3;
use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock, Weak},
};
use winit::window::{CursorGrabMode, Window};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::components::Camera;

static INPUT_STATE: OnceLock<Mutex<InputState>> = OnceLock::new();
static WINDOW_HANDLE: OnceLock<Mutex<Weak<Window>>> = OnceLock::new();

#[derive(Default, Clone, Debug)]
pub struct InputState {
    pub keys_pressed: HashSet<KeyCode>,
    pub keys_just_pressed: HashSet<KeyCode>,
    // pub keys_just_released: HashSet<KeyCode>,
    pub mouse_position: (f32, f32),
    pub mouse_previous_position: (f32, f32),
    pub mouse_delta: (f32, f32), // Relative mouse movement (for locked cursor)
    pub mouse_buttons_pressed: HashSet<MouseButton>,
    pub mouse_buttons_just_pressed: HashSet<MouseButton>,
    // pub mouse_buttons_just_released: HashSet<MouseButton>,
    pub mouse_wheel_delta: f32,

    pub camera: Option<Camera>,
}

impl InputState {
    pub fn initialize() {
        INPUT_STATE.set(Mutex::new(InputState::default())).unwrap();
    }

    pub fn current() -> std::sync::MutexGuard<'static, InputState> {
        INPUT_STATE
            .get()
            .expect("InputState not initialized")
            .lock()
            .unwrap()
    }

    pub fn current_mut() -> std::sync::MutexGuard<'static, InputState> {
        INPUT_STATE
            .get()
            .expect("InputState not initialized")
            .lock()
            .unwrap()
    }

    pub fn update_frame() {
        let mut input_state = INPUT_STATE
            .get()
            .expect("InputState not initialized")
            .lock()
            .unwrap();
        input_state.keys_just_pressed.clear();
        // input_state.keys_just_released.clear();
        input_state.mouse_buttons_just_pressed.clear();
        // input_state.mouse_buttons_just_released.clear();
        input_state.mouse_wheel_delta = 0.0;

        // Update mouse previous position
        input_state.mouse_previous_position = input_state.mouse_position;

        // Reset mouse delta (will be set by MouseMoved event)
        input_state.mouse_delta = (0.0, 0.0);
    }

    pub fn get_mouse_delta() -> (f32, f32) {
        let input_state = Self::current();
        input_state.mouse_delta
    }

    pub fn is_key_pressed(key: KeyCode) -> bool {
        Self::current().keys_pressed.contains(&key)
    }

    pub fn is_key_just_pressed(key: KeyCode) -> bool {
        Self::current().keys_just_pressed.contains(&key)
    }

    pub fn is_mouse_button_pressed(button: MouseButton) -> bool {
        Self::current().mouse_buttons_pressed.contains(&button)
    }

    pub fn is_mouse_button_just_pressed(button: MouseButton) -> bool {
        Self::current().mouse_buttons_just_pressed.contains(&button)
    }

    // pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
    //     Self::current().mouse_buttons_just_released.contains(&button)
    // }

    pub fn get_mouse_world_position() -> Option<Vec3> {
        let input_state = INPUT_STATE
            .get()
            .expect("InputState not initialized")
            .lock()
            .unwrap();
        if let Some(camera) = &input_state.camera {
            let pos_2d = camera.screen_to_world(input_state.mouse_position);
            Some(Vec3::new(pos_2d.x, pos_2d.y, 0.0)) // Z = 0 для совместимости с 2D
        } else {
            None
        }
    }
}

/// Get mouse movement delta since last frame
pub fn get_mouse_delta() -> (f32, f32) {
    InputState::get_mouse_delta()
}

// ===== Cursor Control =====

/// Set the window handle for cursor control (call once at startup)
pub fn set_window_handle(window: &std::sync::Arc<Window>) {
    WINDOW_HANDLE
        .set(Mutex::new(std::sync::Arc::downgrade(window)))
        .ok();
}

/// Show or hide the cursor
pub fn show_cursor(show: bool) {
    if let Some(window_weak) = WINDOW_HANDLE.get() {
        if let Ok(guard) = window_weak.lock() {
            if let Some(window) = guard.upgrade() {
                window.set_cursor_visible(show);
            }
        }
    }
}

/// Lock or unlock the cursor to/from the window
pub fn lock_cursor(lock: bool) {
    if let Some(window_weak) = WINDOW_HANDLE.get() {
        if let Ok(guard) = window_weak.lock() {
            if let Some(window) = guard.upgrade() {
                let grab_mode = if lock {
                    CursorGrabMode::Locked
                } else {
                    CursorGrabMode::None
                };
                let _ = window.set_cursor_grab(grab_mode);
            }
        }
    }
}

/// Convenience function to set cursor mode (useful for FPS games)
pub fn set_cursor_mode(visible: bool, locked: bool) {
    show_cursor(visible);
    lock_cursor(locked);
}

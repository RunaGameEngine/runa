use glam::Vec3;
use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock, Weak},
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::{CursorGrabMode, Window};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::components::Camera;

static INPUT_STATE: OnceLock<Mutex<InputState>> = OnceLock::new();
static WINDOW_HANDLE: OnceLock<Mutex<Weak<Window>>> = OnceLock::new();
static WINDOW_STATE: OnceLock<Mutex<WindowState>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct WindowState {
    pub title: String,
    pub fullscreen: bool,
    pub size: (u32, u32),
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            title: "Runa Game".to_string(),
            fullscreen: false,
            size: (1280, 720),
        }
    }
}

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
        let _ = WINDOW_STATE.set(Mutex::new(WindowState::default()));
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

pub fn initialize_window_state(title: impl Into<String>, fullscreen: bool, size: (u32, u32)) {
    let title = title.into();
    if let Some(state) = WINDOW_STATE.get() {
        if let Ok(mut state) = state.lock() {
            state.title = title;
            state.fullscreen = fullscreen;
            state.size = size;
        }
    }
}

fn with_window<R>(f: impl FnOnce(&Window) -> R) -> Option<R> {
    let window_weak = WINDOW_HANDLE.get()?;
    let guard = window_weak.lock().ok()?;
    let window = guard.upgrade()?;
    Some(f(&window))
}

fn with_window_state<R>(f: impl FnOnce(&mut WindowState) -> R) -> Option<R> {
    let state = WINDOW_STATE.get()?;
    let mut guard = state.lock().ok()?;
    Some(f(&mut guard))
}

pub fn window_title() -> Option<String> {
    let state = WINDOW_STATE.get()?;
    let guard = state.lock().ok()?;
    Some(guard.title.clone())
}

pub fn is_fullscreen() -> Option<bool> {
    let state = WINDOW_STATE.get()?;
    let guard = state.lock().ok()?;
    Some(guard.fullscreen)
}

pub fn window_size() -> Option<(u32, u32)> {
    let state = WINDOW_STATE.get()?;
    let guard = state.lock().ok()?;
    Some(guard.size)
}

pub fn set_window_title(title: impl Into<String>) {
    let title = title.into();
    let _ = with_window_state(|state| {
        state.title = title.clone();
    });
    let _ = with_window(|window| {
        window.set_title(&title);
    });
}

pub fn set_fullscreen(fullscreen: bool) {
    let _ = with_window_state(|state| {
        state.fullscreen = fullscreen;
    });
    let _ = with_window(|window| {
        if fullscreen {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(
                window.current_monitor(),
            )));
        } else {
            window.set_fullscreen(None);
        }
    });
}

pub fn toggle_fullscreen() {
    let current = is_fullscreen().unwrap_or(false);
    set_fullscreen(!current);
}

pub fn set_window_size(width: u32, height: u32) {
    let size = (width.max(1), height.max(1));
    let _ = with_window_state(|state| {
        state.size = size;
    });
    let _ = with_window(|window| {
        let _ = window.request_inner_size(PhysicalSize::new(size.0, size.1));
    });
}

pub fn set_window_position(x: i32, y: i32) {
    let _ = with_window(|window| {
        window.set_outer_position(PhysicalPosition::new(x, y));
    });
}

pub fn move_window_by(dx: i32, dy: i32) {
    let _ = with_window(|window| {
        if let Ok(position) = window.outer_position() {
            window.set_outer_position(PhysicalPosition::new(
                position.x.saturating_add(dx),
                position.y.saturating_add(dy),
            ));
        }
    });
}

pub fn screen_center_position() -> Option<(i32, i32)> {
    with_window(|window| {
        let monitor = window.current_monitor()?;
        let position = monitor.position();
        let size = monitor.size();
        let center_x = position.x.saturating_add((size.width / 2) as i32);
        let center_y = position.y.saturating_add((size.height / 2) as i32);
        Some((center_x, center_y))
    })
    .flatten()
}

pub fn centered_window_position() -> Option<(i32, i32)> {
    with_window(|window| {
        let monitor = window.current_monitor()?;
        let monitor_position = monitor.position();
        let monitor_size = monitor.size();
        let window_size = window.outer_size();

        let x = monitor_position.x.saturating_add(
            ((monitor_size.width as i64 - window_size.width as i64) / 2) as i32,
        );
        let y = monitor_position.y.saturating_add(
            ((monitor_size.height as i64 - window_size.height as i64) / 2) as i32,
        );

        Some((x, y))
    })
    .flatten()
}

pub fn center_window() {
    if let Some((x, y)) = centered_window_position() {
        set_window_position(x, y);
    }
}

/// Show or hide the cursor
pub fn show_cursor(show: bool) {
    let _ = with_window(|window| {
        window.set_cursor_visible(show);
    });
}

/// Lock or unlock the cursor to/from the window
pub fn lock_cursor(lock: bool) {
    let _ = with_window(|window| {
        let grab_mode = if lock {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        let _ = window.set_cursor_grab(grab_mode);
    });
}

/// Convenience function to set cursor mode (useful for FPS games)
pub fn set_cursor_mode(visible: bool, locked: bool) {
    show_cursor(visible);
    lock_cursor(locked);
}

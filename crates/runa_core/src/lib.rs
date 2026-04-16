pub mod audio;
pub mod components;
mod console;
pub mod debug_renderer;
pub mod input;
pub mod ocs;
pub mod systems;

pub use console::Console;
pub use ocs::World;

pub mod input_system {
    pub use crate::input::get_mouse_delta;
    pub use crate::input::initialize_window_state;
    pub use crate::input::is_fullscreen;
    pub use crate::input::lock_cursor;
    pub use crate::input::move_window_by;
    pub use crate::input::set_fullscreen;
    pub use crate::input::set_cursor_mode;
    pub use crate::input::set_window_handle;
    pub use crate::input::set_window_position;
    pub use crate::input::set_window_size;
    pub use crate::input::set_window_title;
    pub use crate::input::show_cursor;
    pub use crate::input::toggle_fullscreen;
    pub use crate::input::InputState as Input;
    pub use crate::input::window_size;
    pub use crate::input::window_title;
    pub use winit::{event::MouseButton, keyboard::KeyCode};
}

pub use glam;
pub use glam::{Mat4, Quat, Vec2, Vec3};

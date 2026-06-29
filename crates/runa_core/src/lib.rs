pub mod audio;
pub mod components;
mod console;
pub mod debug_renderer;
pub mod input;
pub mod ocs;
pub mod registry;
pub mod systems;

pub use components::{SerializedField, SerializedFieldAccess, SerializedFieldValue};
pub use console::{Console, ConsoleCommand};
pub use ocs::World;
pub use registry::{
    ArchetypeKey, ArchetypeMetadata, ArchetypeRegistry, ObjectDef, ObjectDefKey, ObjectDefMetadata,
    ObjectDefName, ObjectDefRegistry, RegisteredTypeKind, RegistrationSource, RunaArchetype,
    RunaComponentType, RunaScriptType, RuntimeRegistry, TypeMetadata, TypeRegistry,
};

pub mod input_system {
    pub use crate::input::bind_action;
    pub use crate::input::center_window;
    pub use crate::input::centered_window_position;
    pub use crate::input::get_mouse_delta;
    pub use crate::input::initialize_window_state;
    pub use crate::input::is_action_just_pressed;
    pub use crate::input::is_action_pressed;
    pub use crate::input::is_fullscreen;
    pub use crate::input::list_action_bindings;
    pub use crate::input::lock_cursor;
    pub use crate::input::move_window_by;
    pub use crate::input::parse_input_binding;
    pub use crate::input::register_action;
    pub use crate::input::register_default_actions;
    pub use crate::input::screen_center_position;
    pub use crate::input::set_cursor_mode;
    pub use crate::input::set_fullscreen;
    pub use crate::input::set_window_handle;
    pub use crate::input::set_window_position;
    pub use crate::input::set_window_size;
    pub use crate::input::set_window_title;
    pub use crate::input::show_cursor;
    pub use crate::input::toggle_fullscreen;
    pub use crate::input::unbind_action;
    pub use crate::input::unbind_action_all;
    pub use crate::input::window_size;
    pub use crate::input::window_title;
    pub use crate::input::InputBinding;
    pub use crate::input::InputState as Input;
    pub use winit::{event::MouseButton, keyboard::KeyCode};
}

pub use glam;
pub use glam::{Mat4, Quat, Vec2, Vec3};

pub mod audio;
pub mod codefirst;
pub mod color;
pub mod components;
mod console;
pub mod debug_renderer;
pub mod input;
pub mod math;
pub mod ocs;
pub mod registry;
pub mod systems;

pub use codefirst::{Bundle, QueryMut, QueryRef};
pub use color::Color;
pub use console::{Console, ConsoleCommand};
pub use math::*;
pub use ocs::World;

pub use glam;
pub use glam::{Mat4, Quat, Vec2, Vec3};
pub use winit::{event::MouseButton, keyboard::KeyCode};

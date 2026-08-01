//! `use runa_engine::prelude::*;` for the most common types.

pub use crate::{
    runa_app::RunaApp,
    runa_app::RunaWindowConfig,
    runa_core::{
        glam::{Mat4, Quat, Vec2, Vec3},
        math::{smooth_damp, smooth_damp_unlimited, smooth_damp_vec3, LerpExt},
    },
    Color, Engine,
};

pub use runa_core::components::{
    ActiveCamera, AlphaMode, AudioListener, AudioSource, Camera, Collider2D, CursorInteractable,
    DirectionalLight, FontId, Material, Mesh, MeshRenderer, PointLight, ProjectionType, Sorting,
    SpriteAnimator, SpriteRenderer, Tilemap, TilemapLayer, TilemapRenderer, Transform, UiRenderer,
    Vertex3D,
};

pub use runa_core::input::{
    bind_action, center_window, get_mouse_delta, get_mouse_position, get_mouse_scroll_delta,
    is_action_just_pressed, is_action_pressed, is_fullscreen, is_mouse_button_just_released,
    register_action, set_cursor_mode, set_fullscreen, set_window_size, set_window_title,
    toggle_fullscreen, window_size, window_title, InputBinding, InputState,
};
pub use runa_core::EventBus;
pub use runa_core::KeyCode;

pub use crate::scene::{SaveData, Scene, SceneDescriptor, SceneManager};

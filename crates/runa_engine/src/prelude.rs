//! `use runa_engine::prelude::*;` for the most common types.

pub use crate::{
    Bundle, Color, Component, Engine, QueryMut, QueryRef,
    runa_app::RunaApp,
    runa_app::RunaWindowConfig,
    runa_core::{
        World,
        glam::{Mat4, Quat, Vec2, Vec3},
        math::{LerpExt, smooth_damp, smooth_damp_unlimited, smooth_damp_vec3},
        ocs::{Object, ObjectHandle, ObjectId, Script, ScriptContext},
    },
};

pub use runa_core::components::{
    ActiveCamera, AlphaMode, AudioListener, AudioSource, Camera, Collider2D,
    CursorInteractable, DirectionalLight, Material, Mesh, MeshRenderer, PointLight,
    ProjectionType, Sorting, SpriteAnimator, SpriteRenderer, Tilemap, TilemapLayer,
    TilemapRenderer, Transform, UiRenderer, Vertex3D,
};

pub use runa_core::input::{
    InputState,
    bind_action, center_window, is_action_just_pressed, is_action_pressed,
    is_fullscreen, register_action, set_cursor_mode, set_fullscreen,
    set_window_size, set_window_title, toggle_fullscreen, window_size, window_title,
    InputBinding, get_mouse_delta, get_mouse_position, get_mouse_scroll_delta,
    is_mouse_button_just_released,
};
pub use runa_core::KeyCode;

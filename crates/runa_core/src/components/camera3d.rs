use glam::{Mat4, Vec3};

/// Camera component (3D perspective)
///
/// # Deprecated
/// Use [`Camera`](crate::components::Camera) instead with `new_perspective()` for 3D cameras.
#[deprecated(note = "Use `Camera::new_perspective()` instead")]
pub struct Camera3D {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32, // radians
    pub near: f32,
    pub far: f32,
    pub viewport_size: (u32, u32),
}

impl Camera3D {
    pub fn matrix(&self) -> Mat4 {
        let aspect = self.viewport_size.0 as f32 / self.viewport_size.1 as f32;
        let proj = Mat4::perspective_rh(self.fov, aspect, self.near, self.far);
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        proj * view
    }
}

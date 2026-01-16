use glam::{Mat4, Vec2};

/// Camera component
#[derive(Debug, Copy, Clone)]
pub struct Camera2D {
    /// Local position.
    /// You can manualy set local position of many components inside your object like this.
    pub position: Vec2,
    /// Scale: 1.0 = 1:1, >1 = increase; <1 = decrease
    pub zoom: f32,
    /// Virtual size/camera render size (for example 320x180)
    pub virtual_size: Vec2,
}

impl Camera2D {
    pub fn new(virtual_width: f32, virtual_height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            virtual_size: Vec2::new(virtual_width, virtual_height),
        }
    }

    pub fn matrix(&self) -> Mat4 {
        // Projection: ортографическая проекция с верхним левым углом (0,0)
        let half_w = self.virtual_size.x * 0.5 / self.zoom;
        let half_h = self.virtual_size.y * 0.5 / self.zoom;

        let left = -half_w;
        let right = half_w;
        let bottom = -half_h;
        let top = half_h;

        // Орто проекция
        let proj = Mat4::orthographic_rh(left, right, bottom, top, -1.0, 1.0);

        // View: смещаем мир относительно камеры
        let view = Mat4::from_translation((-self.position).extend(0.0));

        proj * view
    }
}

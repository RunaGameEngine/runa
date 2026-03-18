use glam::{Mat4, Vec2};

/// Camera component
#[derive(Debug, Copy, Clone, Default)]
pub struct Camera2D {
    /// Local position.
    /// You can manualy set local position of many components inside your object like this.
    pub position: Vec2,
    /// Scale: 1.0 = 1:1, >1 = increase; <1 = decrease
    pub zoom: f32,
    /// Virtual size/camera render size (for example 320 x 180)
    pub virtual_size: Vec2,
    // pub pixel_perfect: bool, // ← новый флаг
    pub viewport_size: (u32, u32),

    aspect_correction: f32, // ← новое поле
}

impl Camera2D {
    pub fn new(vw: f32, vh: f32) -> Self {
        let mut camera = Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            virtual_size: Vec2::new(vw / 10.0, vh / 10.0),
            viewport_size: (1280, 720),
            aspect_correction: 1.0,
        };
        camera.update_aspect_correction();
        camera
    }

    pub fn render_size(&self) -> (u32, u32) {
        if self.virtual_size == Vec2::new(0.0, 0.0) {
            // Use window size directly
            self.viewport_size
        } else {
            // Use virtual size
            (self.virtual_size.x as u32, self.virtual_size.y as u32)
        }
    }

    pub fn scale_factor(&self) -> f32 {
        let (render_width, render_height) = self.render_size();
        let window_width = self.viewport_size.0 as f32;
        let window_height = self.viewport_size.1 as f32;

        // Use uniform scaling (preserve aspect ratio)
        let scale_x = window_width / render_width as f32;
        let scale_y = window_height / render_height as f32;

        scale_x.min(scale_y)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport_size = (width.max(1), height.max(1));
    }

    pub fn matrix(&self) -> Mat4 {
        let (render_width, render_height) = self.render_size();

        // Calculate visible world size based on zoom
        let world_width = render_width as f32 * self.zoom;
        let world_height = render_height as f32 * self.zoom;

        // Create orthographic projection
        // Maps world units to render pixels
        let left = -world_width * 0.5;
        let right = world_width * 0.5;
        let bottom = -world_height * 0.5;
        let top = world_height * 0.5;

        let proj = Mat4::orthographic_rh_gl(left, right, bottom, top, -1000.0, 1000.0);

        // Create view matrix (camera transform)
        let view =
            Mat4::from_translation(Vec2::new(-self.position.x, -self.position.y).extend(0.0));

        // Combine projection and view
        proj * view
    }

    pub fn update_aspect_correction(&mut self) {
        let window_aspect = self.viewport_size.0 as f32 / self.viewport_size.1 as f32;
        let virtual_aspect = self.virtual_size.x / self.virtual_size.y;
        self.aspect_correction = virtual_aspect / window_aspect;
    }

    pub fn screen_to_world(&self, screen_pos: (f32, f32)) -> Vec2 {
        let (screen_x, screen_y) = screen_pos;
        let (viewport_width, viewport_height) = self.viewport_size;

        // Нормализуем к NDC
        let ndc_x = (screen_x / viewport_width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y / viewport_height as f32) * 2.0;

        // ← КЛЮЧЕВОЕ: применяем ОБРАТНУЮ коррекцию аспекта
        let corrected_ndc_x = ndc_x / self.aspect_correction;

        let world_x = corrected_ndc_x * (self.virtual_size.x * 0.5) / self.zoom + self.position.x;
        let world_y = ndc_y * (self.virtual_size.y * 0.5) / self.zoom + self.position.y;

        Vec2::new(world_x, world_y)
    }
}

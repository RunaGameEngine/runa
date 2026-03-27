use glam::{Mat4, Vec2, Vec3};

/// Тип проекции камеры
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    /// Ортографическая проекция (для 2D)
    Orthographic,
    /// Перспективная проекция (для 3D)
    Perspective,
}

/// Универсальный компонент камеры с поддержкой 2D и 3D
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// Позиция камеры в мире
    pub position: Vec3,
    /// Точка, куда смотрит камера (для 3D)
    pub target: Vec3,
    /// Вектор направления "вверх" (для 3D)
    pub up: Vec3,

    /// Тип проекции
    pub projection: ProjectionType,

    // Параметры для ортографической проекции (2D)
    /// Размер ортографической камеры (ширина, высота)
    pub ortho_size: Vec2,
    /// Ближняя плоскость отсечения
    pub near: f32,
    /// Дальняя плоскость отсечения
    pub far: f32,

    // Параметры для перспективной проекции (3D)
    /// Поле зрения в радианах (для 3D)
    pub fov: f32,

    /// Размер области рендеринга (viewport)
    pub viewport_size: (u32, u32),
}

impl Camera {
    /// Создать новую камеру с ортографической проекцией (2D)
    ///
    /// # Arguments
    /// * `width` - Ширина видимой области
    /// * `height` - Высота видимой области
    /// * `viewport_size` - Размер окна рендеринга
    pub fn new_ortho(width: f32, height: f32, viewport_size: (u32, u32)) -> Self {
        Self {
            position: Vec3::ZERO,
            target: Vec3::Z,
            up: Vec3::Y,
            projection: ProjectionType::Orthographic,
            ortho_size: Vec2::new(width / 10.0, height / 10.0),
            near: -1000.0,
            far: 1000.0,
            fov: 0.0, // Не используется для ортографии
            viewport_size,
        }
    }

    /// Создать новую камеру с перспективной проекцией (3D)
    ///
    /// # Arguments
    /// * `position` - Позиция камеры
    /// * `target` - Точка, куда смотрит камера
    /// * `up` - Вектор направления "вверх"
    /// * `fov` - Поле зрения в радианах
    /// * `near` - Ближняя плоскость отсечения
    /// * `far` - Дальняя плоскость отсечения
    /// * `viewport_size` - Размер окна рендеринга
    pub fn new_perspective(
        position: Vec3,
        target: Vec3,
        up: Vec3,
        fov: f32,
        near: f32,
        far: f32,
        viewport_size: (u32, u32),
    ) -> Self {
        Self {
            position,
            target,
            up,
            projection: ProjectionType::Perspective,
            ortho_size: Vec2::ZERO, // Не используется для перспективы
            near,
            far,
            fov,
            viewport_size,
        }
    }

    /// Получить матрицу вида-проекции (view-projection matrix)
    pub fn matrix(&self) -> Mat4 {
        match self.projection {
            ProjectionType::Orthographic => self.ortho_matrix(),
            ProjectionType::Perspective => self.perspective_matrix(),
        }
    }

    /// Получить ортографическую матрицу проекции
    fn ortho_matrix(&self) -> Mat4 {
        let half_width = self.ortho_size.x * 0.5;
        let half_height = self.ortho_size.y * 0.5;

        // orthographic_rh_gl использует Z от -1 до 1 (NDC)
        let proj = Mat4::orthographic_rh_gl(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        );

        let view = Mat4::from_translation(-self.position);

        proj * view
    }

    /// Получить перспективную матрицу проекции
    fn perspective_matrix(&self) -> Mat4 {
        let aspect = self.viewport_size.0 as f32 / self.viewport_size.1 as f32;
        let proj = Mat4::perspective_rh(self.fov, aspect, self.near, self.far);
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        proj * view
    }

    /// Установить позицию камеры
    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    /// Установить размер ортографической камеры
    pub fn set_ortho_size(&mut self, size: Vec2) {
        self.ortho_size = size;
    }

    /// Установить поле зрения (для перспективной камеры)
    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }

    /// Получить аспект (соотношение сторон)
    pub fn aspect(&self) -> f32 {
        self.viewport_size.0 as f32 / self.viewport_size.1 as f32
    }

    /// Обновить размер viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport_size = (width.max(1), height.max(1));
    }

    /// Преобразовать экранные координаты в мировые (для ортографической камеры)
    pub fn screen_to_world(&self, screen_pos: (f32, f32)) -> Vec2 {
        let (screen_x, screen_y) = screen_pos;
        let (viewport_width, viewport_height) = self.viewport_size;

        // Нормализуем к NDC
        let ndc_x = (screen_x / viewport_width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y / viewport_height as f32) * 2.0;

        // Для ортографической камеры
        let half_width = self.ortho_size.x * 0.5;
        let half_height = self.ortho_size.y * 0.5;

        // Учитываем aspect correction как в renderer.rs
        // aspect = (virtual_size.x / virtual_size.y) / (window_width / window_height)
        let virtual_aspect = self.ortho_size.x / self.ortho_size.y;
        let window_aspect = viewport_width as f32 / viewport_height as f32;
        let aspect_correction = virtual_aspect / window_aspect;

        let corrected_ndc_x = ndc_x / aspect_correction;

        let world_x = corrected_ndc_x * half_width + self.position.x;
        let world_y = ndc_y * half_height + self.position.y;

        Vec2::new(world_x, world_y)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new_perspective(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
            75.0_f32.to_radians(),
            0.1,
            1000.0,
            (1280, 720),
        )
    }
}

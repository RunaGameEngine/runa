use glam::Vec3;

#[derive(Default)] // todo: Clone
pub struct CursorInteractable {
    pub is_pressed: bool,
    pub is_hovered: bool,
    pub was_hovered: bool,
    pub bounds_size: Vec3, // половина размера (extents) для проверки попадания курсора
    pub on_click: Option<Box<dyn FnMut() + Send>>,
    pub on_hover_enter: Option<Box<dyn FnMut() + Send>>,
    pub on_hover_exit: Option<Box<dyn FnMut() + Send>>,
}

impl CursorInteractable {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            is_pressed: false,
            is_hovered: false,
            was_hovered: false,
            bounds_size: Vec3::new(width * 0.5, height * 0.5, 0.1), // по умолчанию небольшая глубина
            on_click: None,
            on_hover_enter: None,
            on_hover_exit: None,
        }
    }

    pub fn new_3d(width: f32, height: f32, depth: f32) -> Self {
        Self {
            is_pressed: false,
            is_hovered: false,
            was_hovered: false,
            bounds_size: Vec3::new(width * 0.5, height * 0.5, depth * 0.5),
            on_click: None,
            on_hover_enter: None,
            on_hover_exit: None,
        }
    }

    pub fn contains_point(&self, point: Vec3, center: Vec3) -> bool {
        let min_x = center.x - self.bounds_size.x;
        let max_x = center.x + self.bounds_size.x;
        let min_y = center.y - self.bounds_size.y;
        let max_y = center.y + self.bounds_size.y;
        let min_z = center.z - self.bounds_size.z;
        let max_z = center.z + self.bounds_size.z;

        point.x >= min_x
            && point.x <= max_x
            && point.y >= min_y
            && point.y <= max_y
            && point.z >= min_z
            && point.z <= max_z
    }

    pub fn update_callbacks(&mut self) {
        if self.is_hovered && !self.was_hovered {
            if let Some(ref mut callback) = self.on_hover_enter {
                callback();
            }
        } else if !self.is_hovered && self.was_hovered {
            if let Some(ref mut callback) = self.on_hover_exit {
                callback();
            }
        }
        self.was_hovered = self.is_hovered;
    }

    pub fn set_on_hover_enter<F>(&mut self, callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        self.on_hover_enter = Some(Box::new(callback));
    }

    pub fn set_on_hover_exit<F>(&mut self, callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        self.on_hover_exit = Some(Box::new(callback));
    }
}

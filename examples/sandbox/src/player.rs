use runa_core::{
    components::{ActiveCamera, Camera, SpriteRenderer, Transform},
    glam::Vec3,
    input_system::*,
    ocs::Script,
};

pub struct Player {
    speed: f32,
    direction: Vec3,
}

impl Player {
    pub fn new() -> Self {
        Self {
            speed: 0.25,
            direction: Vec3::ZERO,
        }
    }
}

impl Script for Player {
    fn construct(&self, _object: &mut runa_core::ocs::Object) {
        // конструктор объекта
        _object
            .add_component(Transform::default())
            .add_component(Camera::new_ortho(320.0, 180.0, (1280, 720)))
            .add_component(ActiveCamera)
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::load_image!("assets/art/Charactert.png")), // загрузка спрайта
            });
    }

    fn start(&mut self, _object: &mut runa_core::ocs::Object) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            // стартовая позиция игрока
            transform.position = Vec3::new(0.0, 0.0, 0.0);
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        }
    }

    fn update(&mut self, _object: &mut runa_core::ocs::Object, _dt: f32) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            self.direction = Vec3::ZERO;
            // реализована система ввода с кливиатуры пользователя.
            if Input::is_key_pressed(KeyCode::KeyW) {
                self.direction.y = 1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyS) {
                self.direction.y = -1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyD) {
                self.direction.x = 1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyA) {
                self.direction.x = -1.0;
            }
            // update каждый игровой тик, независимо от fps пользователя
            transform.position += self.direction.normalize_or_zero() * self.speed;
        }
    }
}

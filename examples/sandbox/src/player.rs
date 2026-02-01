use glam::Vec2;
use runa_core::{
    components::{sprite_renderer::SpriteRenderer, transform::Transform},
    ocs::script::Script,
};
use winit::keyboard::KeyCode;

pub struct Player {
    speed: f32,
    direction: Vec2,
}

impl Player {
    pub fn new() -> Self {
        Self {
            speed: 0.25,
            direction: Vec2::ZERO,
        }
    }
}

impl Script for Player {
    fn construct(&self, _object: &mut runa_core::ocs::object::Object) {
        _object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::loader::load_image("assets/Charactert.png")),
            });
    }

    fn input(
        &mut self,
        _object: &mut runa_core::ocs::object::Object,
        _input: &runa_core::input::InputState,
    ) {
        self.direction = Vec2::ZERO;

        if _input.is_key_pressed(KeyCode::KeyW) {
            self.direction.y = 1.0;
        }
        if _input.is_key_pressed(KeyCode::KeyS) {
            self.direction.y = -1.0;
        }
        if _input.is_key_pressed(KeyCode::KeyD) {
            self.direction.x = 1.0;
        }
        if _input.is_key_pressed(KeyCode::KeyA) {
            self.direction.x = -1.0;
        }
    }

    fn start(&mut self, _object: &mut runa_core::ocs::object::Object) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            transform.position = Vec2 { x: 0.0, y: 0.0 };
            transform.scale = Vec2 { x: 1.0, y: 1.0 };
        }
    }

    fn update(&mut self, _object: &mut runa_core::ocs::object::Object, _dt: f32) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            transform.position += self.direction.normalize_or_zero() * self.speed;
        }
    }
}

use glam::Vec3;
use runa_core::{
    components::{SpriteRenderer, Transform},
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
        // Object construction
        _object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::loader::load_image("assets/Charactert.png")), // Load the sprite
                texture_path: Some("assets/Charactert.png".to_string()),
            });
    }

    fn start(&mut self, _object: &mut runa_core::ocs::Object) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            // Initial player position
            transform.position = Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            transform.scale = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            };
        }
    }

    fn update(&mut self, _object: &mut runa_core::ocs::Object, _dt: f32) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            self.direction = Vec3::ZERO;
            // Handle player keyboard input.
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
            // Update on every simulation tick, independent from frame rate
            transform.position += self.direction.normalize_or_zero() * self.speed;
        }
    }
}

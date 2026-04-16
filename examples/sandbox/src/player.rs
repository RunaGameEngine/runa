use runa_asset::load_image;
use runa_core::{
    components::{
        ui::CanvasSpace, ActiveCamera, Camera, Canvas, Collider2D, SpriteRenderer, Transform,
    },
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
        // Object construction
        _object
            .add_component(Transform::default())
            .add_component(Camera::new_ortho(320.0, 180.0, (1280, 720)))
            .add_component(ActiveCamera)
            .add_component(SpriteRenderer::new(Some(load_image!(
                "assets/art/Charactert.png"
            ))))
            .add_component(Collider2D::new(16.0, 16.0))
            .add_component(Canvas::new(CanvasSpace::Camera));
    }

    fn start(&mut self, _object: &mut runa_core::ocs::Object) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            // Initial player position
            transform.position = Vec3::new(0.0, 0.0, 0.0);
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        }
    }

    fn update(&mut self, _object: &mut runa_core::ocs::Object, _dt: f32) {
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

        let Some(current_position) = _object
            .get_component::<Transform>()
            .map(|transform| transform.position)
        else {
            return;
        };

        let movement = self.direction.normalize_or_zero() * self.speed;
        let next_position = current_position + movement;
        let next_position_2d = next_position.truncate();

        if !_object.would_collide_2d_at(next_position_2d) {
            if let Some(transform) = _object.get_component_mut::<Transform>() {
                transform.position = next_position;
            }
        }
    }
}

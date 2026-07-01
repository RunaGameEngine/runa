use runa_core::{
    components::*,
    glam::Vec3,
    input::InputState,
    ocs::{Object, Script, ScriptContext},
};
use winit::keyboard::KeyCode;

pub struct Player {
    speed: f32,
    direction: Vec3,
}

impl Player {
    pub fn new() -> Self {
        Self {
            speed: 6.0,
            direction: Vec3::ZERO,
        }
    }
}

impl Script for Player {
    fn start(&mut self, _ctx: &mut ScriptContext) {}

    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        self.direction = Vec3::ZERO;
        if InputState::is_key_pressed(KeyCode::KeyW) {
            self.direction.y = 1.0;
        }
        if InputState::is_key_pressed(KeyCode::KeyS) {
            self.direction.y = -1.0;
        }
        if InputState::is_key_pressed(KeyCode::KeyD) {
            self.direction.x = 1.0;
        }
        if InputState::is_key_pressed(KeyCode::KeyA) {
            self.direction.x = -1.0;
        }

        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position += self.direction.normalize_or_zero() * self.speed * dt;
        }
    }
}

pub fn create_player() -> Object {
    Object::new("Player")
        .with(AudioListener::new())
        .with(Camera::new_orthographic(32.0, 18.0))
        .with(ActiveCamera)
        .with(SpriteRenderer::from_path("assets/art/Charactert.png"))
        .with(Player::new())
}

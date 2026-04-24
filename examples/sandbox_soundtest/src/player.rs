use runa_asset::load_image;
use runa_core::{
    components::{ActiveCamera, AudioListener, Camera, SpriteRenderer, Transform},
    glam::Vec3,
    input_system::*,
    ocs::{Object, Script, ScriptContext, World},
    SerializedFieldAccess,
};
use runa_engine::RunaArchetype;

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

impl SerializedFieldAccess for Player {}

impl Script for Player {
    fn start(&mut self, ctx: &mut ScriptContext) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position = Vec3::new(0.0, 0.0, 0.0);
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        }
    }

    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        self.direction = Vec3::ZERO;
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

        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position += self.direction.normalize_or_zero() * self.speed * dt;
        }
    }
}

pub fn create_player() -> Object {
    Object::new("Player")
        .with(AudioListener::new())
        .with(Camera::new_ortho(32.0, 18.0))
        .with(ActiveCamera)
        .with(SpriteRenderer {
            texture: Some(load_image!("assets/art/Charactert.png")),
            texture_path: Some("assets/Charactert.png".to_string()),
            // Sound-test keeps the default 16 PPU sprite convention used by the
            // bundled 2D examples.
            pixels_per_unit: 16.0,
            uv_rect: SpriteRenderer::FULL_UV_RECT,
        })
        .with(Player::new())
}

#[derive(RunaArchetype)]
#[runa(name = "player")]
pub struct PlayerArchetype;

impl PlayerArchetype {
    pub fn create(world: &mut World) -> u64 {
        world.spawn(create_player())
    }
}

use runa_app::{RunaApp, RunaWindowConfig};
use runa_core::World;

use runa_core::Vec3;
use runa_core::{
    components::{SpriteRenderer, Transform},
    input_system::*,
    ocs::Script,
};

mod tester1;
mod tilemap_tester;

use tilemap_tester::TilemapTester;

fn main() {
    // Create a new empty world to hold game objects and systems
    let mut world = World::default();

    // Spawn the player object (managed via its Script implementation)
    world.spawn(Box::new(TilemapTester::new()));
    world.spawn(Box::new(tester1::RotatingSprite1::new()));
    world.spawn(Box::new(Player::new()));

    // Configure the application window
    let config = RunaWindowConfig {
        title: "Runa Sandbox".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
    };

    // Launch the engine with the configured world and window settings
    let _ = RunaApp::run_with_config(world, config);
}

/// Player script — defines behavior for the player-controlled character.
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
    /// Called once when the object is created.
    /// Initializes components (transform + sprite).
    fn construct(&self, _object: &mut runa_core::ocs::Object) {
        _object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::loader::load_image("assets/Charactert.png")),
            });
    }

    /// Called once on the first tick after the object is added to the world.
    /// Sets initial position and scale.
    fn start(&mut self, _object: &mut runa_core::ocs::Object) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            transform.position = Vec3::new(0.0, 0.0, 0.0);
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        }
    }

    /// Called every tick. Handles input and updates position.
    fn update(&mut self, _object: &mut runa_core::ocs::Object, _dt: f32) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            // Reset movement direction
            self.direction = Vec3::ZERO;

            // Read input state (WASD keys)
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

            // Apply normalized movement (diagonal speed compensation)
            transform.position += self.direction.normalize_or_zero() * self.speed;
        }
    }
}

// #![windows_subsystem = "windows"]

use runa_app::{RunaApp, RunaWindowConfig};
use runa_core::World;

use runa_core::components::{ActiveCamera, AudioListener, Camera};
use runa_core::Vec3;
use runa_core::{
    components::{SpriteRenderer, Transform},
    input_system::*,
    ocs::Script,
};

mod sound_emitter;
mod tester1;
mod tilemap_tester;

use tilemap_tester::TilemapTester;

fn main() {
    // Create a new empty world to hold game objects and systems
    let mut world = World::default();

    // Spawn the objects (managed via its Script implementation)
    world.spawn(Box::new(TilemapTester::new()));
    world.spawn(Box::new(tester1::RotatingSprite1::new()));
    world.spawn(Box::new(Player::new()));

    // Spawn 3D sound emitters at different positions
    // Left emitter (-5, 0) - sound will come from left when player is at center
    let test_sound = runa_asset::load_audio!("assets/audio/test.ogg");
    world.spawn(Box::new(sound_emitter::SoundEmitter::new(
        test_sound.clone(),
        Vec3::new(-5.0, 0.0, 0.0),
        "LEFT EMITTER",
    )));

    // // Right emitter (5, 0) - sound will come from right when player is at center
    // world.spawn(Box::new(sound_emitter::SoundEmitter::new(
    //     test_sound.clone(),
    //     Vec3::new(5.0, 0.0, 0.0),
    //     "RIGHT EMITTER",
    // )));

    // Configure the application window
    let config = RunaWindowConfig {
        title: "Runa Sandbox - 3D Audio Test (WASD to move, listen to left/right emitters)"
            .to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    // Launch the engine with the configured world and window settings
    let _ = RunaApp::run_with_config(world, config);
}

/// Player script — defines behavior for the player-controlled character.
pub struct Player {
    speed: f32,
}

impl Player {
    pub fn new() -> Self {
        Self { speed: 0.25 }
    }
}

impl Script for Player {
    /// Called once when the object is created.
    /// Initializes components (transform + sprite).
    fn construct(&self, _object: &mut runa_core::ocs::Object) {
        _object
            .add_component(Transform::default())
            .add_component(AudioListener::new()) // Player hears 3D sounds
            .add_component(Camera::new_ortho(320.0, 180.0, (1280, 720)))
            .add_component(ActiveCamera)
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::load_image!("assets/art/Charactert.png")),
                texture_path: Some("assets/art/Charactert.png".to_string()),
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
        // Read input state (WASD keys)
        let mut direction = Vec3::ZERO;
        if Input::is_key_pressed(KeyCode::KeyW) {
            direction.y = 1.0;
        }
        if Input::is_key_pressed(KeyCode::KeyS) {
            direction.y = -1.0;
        }
        if Input::is_key_pressed(KeyCode::KeyD) {
            direction.x = 1.0;
        }
        if Input::is_key_pressed(KeyCode::KeyA) {
            direction.x = -1.0;
        }

        // Update transform
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            // Apply normalized movement (diagonal speed compensation)
            transform.position += direction.normalize_or_zero() * self.speed;
        }
    }
}

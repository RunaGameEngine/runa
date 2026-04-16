// #![windows_subsystem = "windows"]

use runa_app::{RunaApp, RunaWindowConfig};
use runa_core::World;

mod player;
mod collider_demo;
mod tester1;
mod tilemap_tester;

use collider_demo::ColliderDemoBox;
use tilemap_tester::TilemapTester;

fn main() {
    // Create a new empty world to hold game objects and systems
    let mut world = World::default();

    // Spawn the objects (managed via its Script implementation)
    world.spawn(Box::new(TilemapTester::new()));
    world.spawn(Box::new(tester1::RotatingSprite1::new()));
    world.spawn(Box::new(ColliderDemoBox::new()));
    world.spawn(Box::new(player::Player::new()));

    // Configure the application window
    let config = RunaWindowConfig {
        title: "Runa Sandbox".to_string(),
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

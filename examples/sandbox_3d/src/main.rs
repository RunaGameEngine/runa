// Uncomment to disable console in build
// #![windows_subsystem = "windows"]

use runa_app::{RunaApp, RunaWindowConfig};
use runa_core::input_system;
use runa_core::World;

mod camera_controller;
mod rotating_cube;
mod rotating_cube2;

fn main() {
    // Create a new empty world to hold game objects and systems
    let mut world = World::default();

    // Spawn 3D camera with controller
    world.spawn(Box::new(camera_controller::CameraController::new()));

    // Spawn rotating 3D cube (sprite for now)
    world.spawn(Box::new(rotating_cube::RotatingCube::new()));
    world.spawn(Box::new(rotating_cube2::RotatingCube2::new()));

    // Configure the application window
    let config = RunaWindowConfig {
        title: "Runa 3D Sandbox - WASD to move, Space/Ctrl for up/down, Right-Click to look"
            .to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    // Launch the engine
    let _ = RunaApp::run_with_config(world, config);

    // Restore cursor on exit
    input_system::show_cursor(true);
    input_system::lock_cursor(false);
}

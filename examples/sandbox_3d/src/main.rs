// Uncomment to disable console in build
// #![windows_subsystem = "windows"]

use runa_core::input_system;
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine,
};

mod camera_controller;
mod rotating_cube;
mod rotating_cube2;

fn main() {
    let mut engine = Engine::new();
    engine.register_archetype::<camera_controller::CameraControllerArchetype>();
    engine.register_archetype::<rotating_cube::RotatingCubeArchetype>();
    engine.register_archetype::<rotating_cube2::RotatingCube2Archetype>();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();
        let _ = world.spawn_archetype::<camera_controller::CameraControllerArchetype>();
        let _ = world.spawn_archetype::<rotating_cube::RotatingCubeArchetype>();
        let _ = world.spawn_archetype::<rotating_cube2::RotatingCube2Archetype>();
    }

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

    let _ = RunaApp::run_with_config(world_rc, config);

    input_system::show_cursor(true);
    input_system::lock_cursor(false);
}

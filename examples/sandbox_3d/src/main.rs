use runa_core::input;
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine,
};

mod camera_controller;
mod rotating_cube;
mod rotating_cube2;

fn main() {
    let engine = Engine::new();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();
        world.spawn_object(camera_controller::create_camera_controller());
        world.spawn_object(rotating_cube::create_rotating_cube());
        world.spawn_object(rotating_cube2::create_rotating_cube2());
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

    input::show_cursor(true);
    input::lock_cursor(false);
}

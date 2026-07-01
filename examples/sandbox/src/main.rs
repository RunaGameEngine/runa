// #![windows_subsystem = "windows"]

use runa_core::{
    components::{BackgroundMode, WorldAtmosphere},
    Color,
};
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine,
};

mod collider_demo;
mod player;
mod tester1;
mod tilemap_tester;

fn main() {
    let engine = Engine::new();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();
        world.set_atmosphere(WorldAtmosphere {
            ambient_color: Color::BLACK,
            ambient_intensity: 1.0,
            background_intensity: 1.0,
            background: BackgroundMode::SolidColor {
                color: Color::rgb(0.5, 0.5, 0.5),
            },
        });
        world.spawn_object(tilemap_tester::create_tilemap_tester());
        world.spawn_object(tester1::create_rotating_sprite());
        world.spawn_object(collider_demo::create_collider_demo_box());
        world.spawn_object(player::create_player());
        world.spawn_object(player::create_player_camera());
    }

    let config = RunaWindowConfig {
        title: "Runa Sandbox".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(world_rc, config);
}

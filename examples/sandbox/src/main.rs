// #![windows_subsystem = "windows"]

use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine,
};

mod collider_demo;
mod player;
mod tester1;
mod tilemap_tester;

fn register_game_types(engine: &mut Engine) {
    player::register_types(engine);
    engine.register_archetype::<tilemap_tester::TilemapTesterArchetype>();
    engine.register_archetype::<tester1::RotatingSpriteArchetype>();
    engine.register_archetype::<collider_demo::ColliderDemoBoxArchetype>();
}

fn main() {
    let mut engine = Engine::new();
    register_game_types(&mut engine);
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();

        let _ = world.spawn_archetype::<tilemap_tester::TilemapTesterArchetype>();
        let _ = world.spawn_archetype::<tester1::RotatingSpriteArchetype>();
        let _ = world.spawn_archetype::<collider_demo::ColliderDemoBoxArchetype>();
        let _ = world.spawn_archetype::<player::PlayerArchetype>();
        let _ = world.spawn_archetype::<player::PlayerCameraArchetype>();
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

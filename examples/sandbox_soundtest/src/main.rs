// #![windows_subsystem = "windows"]

use runa_core::Vec3;
use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    Engine,
};

mod player;
mod sound_emitter;
mod tester1;
mod tilemap_tester;

fn main() {
    let mut engine = Engine::new();
    engine.register_archetype::<tilemap_tester::TilemapTesterArchetype>();
    engine.register_archetype::<tester1::RotatingSpriteArchetype>();
    engine.register_archetype::<player::PlayerArchetype>();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();
        let _ = world.spawn_archetype::<tilemap_tester::TilemapTesterArchetype>();
        let _ = world.spawn_archetype::<tester1::RotatingSpriteArchetype>();
        let _ = world.spawn_archetype::<player::PlayerArchetype>();

        // Explicit object composition: data first, behavior attached where needed.
        let test_sound = runa_asset::load_audio!("assets/audio/test.ogg");
        world.spawn(sound_emitter::create_sound_emitter(
            test_sound.clone(),
            Vec3::new(-5.0, 0.0, 0.0),
            "LEFT EMITTER",
        ));
    }

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

    let _ = RunaApp::run_with_config(world_rc, config);
}

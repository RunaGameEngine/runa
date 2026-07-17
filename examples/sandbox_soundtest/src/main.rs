use runa_engine::runa_app::{RunaApp, RunaWindowConfig};
use runa_engine::runa_core::components::{AudioListener, AudioSource, Camera, Transform};
use runa_engine::runa_core::input::InputState;
use runa_engine::runa_ecs;
use runa_engine::system;
use winit::keyboard::KeyCode;

#[system]
fn toggle_sound(world: &mut runa_ecs::World) {
    if InputState::is_key_just_pressed(KeyCode::Space) {
        for (_, source) in world.query_mut::<runa_ecs::W<AudioSource>>() {
            if source.playing {
                source.stop();
            } else {
                source.play();
            }
        }
    }
}

fn main() {
    let mut world = runa_ecs::World::new();

    world.spawn((Camera::new_orthographic(320.0, 180.0),));
    world.spawn((AudioListener::new(), Transform::default()));

    let audio_asset = runa_asset::load_audio!("assets/audio/test.ogg");
    let mut source = AudioSource::with_asset(audio_asset);
    source.looped = true;
    source.play();

    world.spawn((source,));

    let config = RunaWindowConfig {
        title: "Runa Sound Test — Space to toggle".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_world(config, world);
}

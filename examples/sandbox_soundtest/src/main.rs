use runa_engine::runa_app::{RunaApp, RunaWindowConfig};
use runa_engine::runa_core::audio::AudioEngine;
use runa_engine::runa_core::components::{AudioListener, AudioSource, Camera, Transform};
use runa_engine::runa_core::input::InputState;
use runa_engine::runa_ecs;
use runa_engine::system;
use std::sync::{Mutex, OnceLock};
use winit::keyboard::KeyCode;

fn audio_engine() -> &'static Mutex<Option<AudioEngine>> {
    static ENGINE: OnceLock<Mutex<Option<AudioEngine>>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut engine = AudioEngine::new();
        match engine.initialize() {
            Ok(()) => {
                engine.set_master_volume(0.5);
                Mutex::new(Some(engine))
            }
            Err(e) => {
                eprintln!("AudioSystem: failed to initialize audio: {}", e);
                Mutex::new(None)
            }
        }
    })
}

#[system]
fn audio_system(world: &mut runa_ecs::World) {
    let mut guard = audio_engine().lock().unwrap();
    let Some(engine) = guard.as_mut() else { return };

    for (_, source) in world.query_mut::<runa_ecs::W<AudioSource>>() {
        if source.play_requested {
            source.sound_id = engine.play(source);
            source.play_requested = false;
            source.playing = source.sound_id.is_some();
        }
        if source.stop_requested {
            if let Some(id) = source.sound_id {
                engine.stop(id);
            }
            source.sound_id = None;
            source.stop_requested = false;
            source.playing = false;
        }
    }

    for (_, (listener, transform)) in world.query::<(runa_ecs::R<AudioListener>, runa_ecs::R<Transform>)>() {
        if listener.active {
            engine.set_listener(transform.position, transform.rotation, listener.volume);
        }
    }

    engine.update_spatial_volumes();
    engine.cleanup();
}

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

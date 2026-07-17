use runa_core::audio::AudioEngine;
use runa_core::components::{
    AudioListener, AudioSource, CursorInteractable, SpriteAnimator, SpriteRenderer, Transform,
};
use runa_core::input::InputState;
use runa_core::systems::event_system::EventBus;
use runa_core::MouseButton;
use runa_ecs::{R, W};
use runa_macros::system;
use std::sync::{Mutex, OnceLock};

fn audio_engine() -> &'static Mutex<Option<AudioEngine>> {
    static ENGINE: OnceLock<Mutex<Option<AudioEngine>>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut e = AudioEngine::new();
        match e.initialize() {
            Ok(()) => {
                e.set_master_volume(0.5);
                Mutex::new(Some(e))
            }
            Err(err) => {
                eprintln!("audio_system: failed to initialize: {}", err);
                Mutex::new(None)
            }
        }
    })
}

#[system("crate")]
pub fn cursor_interaction(world: &mut runa_ecs::World) {
    let world_pos = match InputState::get_mouse_world_position() {
        Some(p) => p,
        None => return,
    };
    let mouse_down = InputState::is_mouse_button_just_pressed(MouseButton::Left);

    for (_, (interactable, transform)) in world.query_mut::<(W<CursorInteractable>, R<Transform>)>()
    {
        interactable.is_hovered = interactable.contains_point(world_pos, transform.position);
        if mouse_down && interactable.is_hovered {
            if let Some(cb) = interactable.on_click_mut() {
                if let Ok(f) = cb.get_mut() {
                    f();
                }
            }
        }
        interactable.update_callbacks();
    }
}

#[system("crate")]
pub fn audio_system(world: &mut runa_ecs::World) {
    let mut guard = audio_engine().lock().unwrap();
    let Some(engine) = guard.as_mut() else {
        return;
    };

    for (_, source) in world.query_mut::<W<AudioSource>>() {
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

    for (_, (listener, transform)) in world.query::<(R<AudioListener>, R<Transform>)>() {
        if listener.active {
            engine.set_listener(transform.position, transform.rotation, listener.volume);
        }
    }

    engine.update_spatial_volumes();
    engine.cleanup();
}

#[system("crate")]
pub fn eventbus_system(world: &mut runa_ecs::World) {
    for (_, bus) in world.query_mut::<W<EventBus>>() {
        bus.process();
    }
}

#[system("crate")]
pub fn sprite_animator_system(world: &mut runa_ecs::World) {
    let dt = 1.0 / 60.0;
    for (_, (animator, sprite)) in world.query_mut::<(W<SpriteAnimator>, W<SpriteRenderer>)>() {
        let uv = animator.tick(dt);
        sprite.uv_rect = uv;
    }
}

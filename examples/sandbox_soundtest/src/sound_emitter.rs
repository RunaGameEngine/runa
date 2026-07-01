use runa_asset::AudioAsset;
use runa_core::components::*;
use runa_core::glam::Vec3;
use runa_core::ocs::{Script, ScriptContext};
use std::sync::Arc;

pub struct SoundEmitter {
    label: &'static str,
}

impl SoundEmitter {
    pub fn new(label: &'static str) -> Self {
        Self { label }
    }
}

impl Script for SoundEmitter {
    fn update(&mut self, ctx: &mut ScriptContext, _dt: f32) {
        if let Some(transform) = ctx.get_component::<Transform>() {
            let distance = transform.position.length();
            if distance < 3.0 {
                println!("[{}] NEAR (distance: {:.1})", self.label, distance);
            } else if distance < 10.0 {
                println!("[{}] MEDIUM (distance: {:.1})", self.label, distance);
            } else {
                println!("[{}] FAR (distance: {:.1})", self.label, distance);
            }
        }
    }
}

pub fn create_sound_emitter(
    audio_asset: Arc<AudioAsset>,
    position: Vec3,
    label: &'static str,
) -> (Transform, AudioSource, SpriteRenderer, SoundEmitter) {
    let mut audio = AudioSource::with_asset_3d(audio_asset);
    audio.source_path = Some("assets/audio/test.ogg".to_string());
    audio.looped = true;
    audio.play_on_awake = true;
    audio.spatial = true;
    audio.min_distance = 2.0;
    audio.max_distance = 20.0;

    let mut transform = Transform::default();
    transform.position = position;

    (
        transform,
        audio,
        SpriteRenderer::from_path("assets/art/Tester1.png"),
        SoundEmitter::new(label),
    )
}

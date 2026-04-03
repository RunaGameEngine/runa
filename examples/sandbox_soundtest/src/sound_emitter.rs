use runa_asset::AudioAsset;
use runa_core::components::{AudioSource, SpriteRenderer, Transform};
use runa_core::glam::Vec3;
use runa_core::ocs::{Object, Script};
use std::sync::Arc;

/// Sound emitter — plays 3D spatial audio at a fixed position in the world
///
/// This script creates a sound source that:
/// - Plays continuously (looped)
/// - Has a visual indicator (sprite)
/// - Shows distance to player in debug output
pub struct SoundEmitter {
    audio_asset: Arc<AudioAsset>,
    spawn_position: Vec3,
    label: &'static str,
}

impl SoundEmitter {
    pub fn new(audio_asset: Arc<AudioAsset>, position: Vec3, label: &'static str) -> Self {
        Self {
            audio_asset,
            spawn_position: position,
            label,
        }
    }
}

impl Script for SoundEmitter {
    fn construct(&self, object: &mut Object) {
        // Create 3D audio source with play_on_awake
        let mut audio = AudioSource::with_asset_3d(self.audio_asset.clone());
        audio.source_path = Some("assets/audio/test.ogg".to_string());
        audio.looped = true;
        audio.play_on_awake = true;
        audio.spatial = true;
        audio.min_distance = 2.0; // Full volume within 2 units
        audio.max_distance = 20.0; // Silent beyond 20 units

        object
            .add_component(Transform::default())
            .add_component(audio)
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::load_image!("assets/art/Tester1.png")),
                texture_path: Some("assets/art/Tester1.png".to_string()),
            });
    }

    fn start(&mut self, object: &mut Object) {
        if let Some(transform) = object.get_component_mut::<Transform>() {
            transform.position = self.spawn_position;
            println!("🔊 [{}] spawned at {:?}", self.label, self.spawn_position);
        }
    }

    fn update(&mut self, object: &mut Object, _dt: f32) {
        // Optional: Print distance to player for debugging
        // This helps verify 3D audio is working
        if let Some(transform) = object.get_component::<Transform>() {
            // Player is typically at (0, 0) at start
            let distance = transform.position.length();

            // Print distance every 60 frames (approx 1 second)
            // In a real game you'd use a timer
            if distance < 3.0 {
                println!("🔊 [{}] NEAR (distance: {:.1})", self.label, distance);
            } else if distance < 10.0 {
                println!("🔊 [{}] MEDIUM (distance: {:.1})", self.label, distance);
            } else {
                println!("🔊 [{}] FAR (distance: {:.1})", self.label, distance);
            }
        }
    }
}

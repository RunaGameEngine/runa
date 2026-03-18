use crate::components::AudioSource;
use rodio::source::Source;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub usize);

/// Audio engine resource — manages all audio playback
pub struct AudioEngine {
    stream: Option<MixerDeviceSink>,
    sinks: HashMap<SoundId, Arc<Player>>,
    next_id: usize,
    master_volume: f32,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            stream: None,
            sinks: HashMap::new(),
            next_id: 0,
            master_volume: 1.0,
        }
    }

    /// Initialize audio output (call once at startup)
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = DeviceSinkBuilder::open_default_sink()?;
        stream.log_on_drop(false); // Suppress the drop warning
        self.stream = Some(stream);
        Ok(())
    }

    /// Play audio source (with optimized caching)
    pub fn play(&mut self, audio_source: &AudioSource) -> Option<SoundId> {
        let stream = self.stream.as_ref()?;
        let asset = audio_source.audio_asset.as_ref()?;

        // Create player
        let player = Player::connect_new(stream.mixer());
        player.set_volume(audio_source.volume * self.master_volume);

        // Create source from cached PCM samples (FAST!)
        let source = asset.create_source();

        // Loop if needed
        if audio_source.looped {
            player.append(source.repeat_infinite());
        } else {
            player.append(source);
        }

        // Store player
        let id = SoundId(self.next_id);
        self.next_id += 1;
        self.sinks.insert(id, Arc::new(player));

        Some(id)
    }

    /// Stop sound by ID
    pub fn stop(&mut self, id: SoundId) {
        if let Some(player) = self.sinks.remove(&id) {
            player.stop();
        }
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Cleanup finished sounds
    pub fn cleanup(&mut self) {
        self.sinks.retain(|_, player| !player.empty());
    }

    /// Get number of active sounds
    pub fn active_sounds(&self) -> usize {
        self.sinks.len()
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

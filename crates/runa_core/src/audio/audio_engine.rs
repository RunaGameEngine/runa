//! Simple audio module using rodio 0.20
//!
//! This module provides basic audio playback functionality
//! using the rodio crate.

use rodio::source::Source;
use rodio::Decoder;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use crate::components::AudioSource;

/// Unique identifier for a playing sound
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub usize);

/// Audio engine that manages all sound playback
pub struct AudioEngine {
    #[allow(dead_code)]
    stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
    sinks: HashMap<SoundId, Arc<rodio::Sink>>,
    next_id: usize,
    master_volume: f32,
}

impl AudioEngine {
    /// Creates a new audio engine
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, handle) = rodio::OutputStream::try_default()?;

        Ok(Self {
            stream,
            handle,
            sinks: HashMap::new(),
            next_id: 0,
            master_volume: 1.0,
        })
    }

    /// Plays a sound and returns its ID
    pub fn play(&mut self, audio_source: &AudioSource) -> Option<SoundId> {
        let sink = rodio::Sink::try_new(&self.handle).ok()?;

        // Decode the audio data
        let cursor = Cursor::new((*audio_source.sound_data).clone());
        let source = Decoder::new(cursor).ok()?;

        // Set volume
        sink.set_volume(audio_source.volume);

        // Set looping
        if audio_source.looped {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        // Store the sink
        let id = SoundId(self.next_id);
        self.next_id += 1;
        self.sinks.insert(id, Arc::new(sink));

        Some(id)
    }

    /// Stops a playing sound by its ID
    pub fn stop(&mut self, id: SoundId) {
        if let Some(sink) = self.sinks.remove(&id) {
            sink.stop();
        }
    }

    /// Sets the volume of a playing sound
    pub fn set_sound_volume(&mut self, id: SoundId, volume: f32) {
        if let Some(sink) = self.sinks.get(&id) {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    /// Sets the master volume for all sounds
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        for sink in self.sinks.values() {
            sink.set_volume(self.master_volume);
        }
    }

    /// Checks if a sound is still playing
    pub fn is_playing(&self, id: SoundId) -> bool {
        if let Some(sink) = self.sinks.get(&id) {
            !sink.empty()
        } else {
            false
        }
    }

    /// Cleans up finished sounds
    pub fn cleanup(&mut self) {
        self.sinks.retain(|_, sink| !sink.empty());
    }

    /// Gets the number of currently playing sounds
    pub fn active_sound_count(&self) -> usize {
        self.sinks.len()
    }
}

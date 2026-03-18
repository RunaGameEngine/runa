use runa_asset::AudioAsset;
use std::sync::Arc;

/// Audio source component — attaches audio to a game object
#[derive(Clone)]
pub struct AudioSource {
    /// Cached audio asset (decoded PCM samples)
    pub audio_asset: Option<Arc<AudioAsset>>,
    /// Playback volume (0.0 to 1.0)
    pub volume: f32,
    /// Loop playback
    pub looped: bool,
    /// Is currently playing
    pub playing: bool,
}

impl AudioSource {
    /// Create empty audio source (2D sound)
    pub fn new2d() -> Self {
        Self {
            audio_asset: None,
            volume: 1.0,
            looped: false,
            playing: false,
        }
    }

    /// Create audio source with pre-loaded asset
    pub fn with_asset(audio_asset: Arc<AudioAsset>) -> Self {
        Self {
            audio_asset: Some(audio_asset),
            volume: 1.0,
            looped: false,
            playing: false,
        }
    }

    /// Set audio asset
    pub fn set_asset(&mut self, audio_asset: Arc<AudioAsset>) {
        self.audio_asset = Some(audio_asset);
    }
}

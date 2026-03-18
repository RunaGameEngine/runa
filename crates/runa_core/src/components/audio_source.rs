use std::sync::Arc;

pub struct AudioSource {
    pub sound_data: Arc<Vec<u8>>,
    /// min = 0.0, max = 1.0
    pub volume: f32,
    pub is_3d: bool,
    pub looped: bool,
}

impl AudioSource {
    pub fn new2d(sound_data: Vec<u8>) -> Self {
        Self {
            sound_data: Arc::new(sound_data),
            volume: 1.0,
            looped: false,
            is_3d: false,
        }
    }
    pub fn new3d(sound_data: Vec<u8>) -> Self {
        Self {
            sound_data: Arc::new(sound_data),
            volume: 1.0,
            looped: false,
            is_3d: true,
        }
    }

    /// Sets the volume (0.0 to 1.0)
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Sets whether the sound should loop
    pub fn with_loop(mut self, looped: bool) -> Self {
        self.looped = looped;
        self
    }
}

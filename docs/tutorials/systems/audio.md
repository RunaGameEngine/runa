# Audio System

Learn how to add sound effects and background music to your game.

## Quick Start

### Step 1: Add AudioSource Component

Add an `AudioSource` component to your object:

```rust
use runa_core::components::AudioSource;

object.add_component(AudioSource::with_asset(
    runa_asset::load_audio!("assets/sound.ogg")
));
```

### Step 2: Play Sounds

Play sounds in your script's `update()` method:

```rust
fn update(&mut self, object: &mut Object, dt: f32, world: &mut World) {
    if Input::is_key_just_pressed(KeyCode::KeyV) {
        // Get the audio asset from the component
        if let Some(audio_source) = object.get_component::<AudioSource>() {
            if let Some(asset) = &audio_source.audio_asset {
                // Create a one-shot audio source
                let one_shot = AudioSource {
                    audio_asset: Some(asset.clone()),
                    volume: 1.0,
                    looped: false,
                    playing: true,
                };
                // Play the sound
                world.play_sound(&one_shot);
            }
        }
    }
}
```

## Loading Audio Files

Use the `load_audio!` macro to load sounds:

```rust
// Load an audio file (OGG or WAV format)
let sound = runa_asset::load_audio!("assets/jump.ogg");
```

## AudioSource Properties

| Property | Type | Description |
|----------|------|-------------|
| `audio_asset` | `Option<Arc<AudioAsset>>` | The loaded sound data |
| `volume` | `f32` | Playback volume (0.0 to 1.0) |
| `looped` | `bool` | Whether to loop the sound |
| `playing` | `bool` | Is currently playing |

## Creating AudioSource

```rust
// With a pre-loaded asset
let audio = runa_asset::load_audio!("assets/sound.ogg");
let source = AudioSource::with_asset(audio);

// Empty source (set asset later)
let source = AudioSource::new2d();

// Custom volume
let mut source = AudioSource::with_asset(audio);
source.volume = 0.5; // 50% volume
```

## Complete Example: Jump Sound

```rust
use runa_core::{
    components::{AudioSource, Transform},
    input_system::*,
    ocs::{Object, Script},
    World,
    glam::Vec3,
};

pub struct Player;

impl Player {
    pub fn new() -> Self {
        Self
    }
}

impl Script for Player {
    fn construct(&self, object: &mut Object) {
        object
            .add_component(Transform::default())
            .add_component(AudioSource::with_asset(
                runa_asset::load_audio!("assets/jump.ogg")
            ));
    }

    fn update(&mut self, object: &mut Object, _dt: f32, world: &mut World) {
        // Play jump sound when space is pressed
        if Input::is_key_just_pressed(KeyCode::Space) {
            if let Some(audio_source) = object.get_component::<AudioSource>() {
                if let Some(asset) = &audio_source.audio_asset {
                    let one_shot = AudioSource {
                        audio_asset: Some(asset.clone()),
                        volume: 1.0,
                        looped: false,
                        playing: true,
                    };
                    world.play_sound(&one_shot);
                }
            }
        }
    }
}
```

## Supported Formats

- **OGG** (recommended) - Good compression, open format
- **WAV** - Uncompressed, larger file size

## Tips

- Use OGG for most sounds (smaller file sizes)
- Keep sound files under 1 MB for quick loading
- Use `load_audio!` macro for automatic caching
- Sounds are played asynchronously (won't block your game)

## Next Steps

- [Input](../systems/input.md) for triggering sounds
- [Scripts](../scripts/creating-scripts.md) for game logic

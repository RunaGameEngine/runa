# Creating Your First App

This tutorial shows you how to create a basic Runa Engine application.

## Step 1: Set Up Your Project

In your `Cargo.toml`, add the Runa Engine crates:

```toml
[dependencies]
runa_engine = { git = "https://github.com/AnuranGames/runa-engine.git", tag = "<current version>" }
```
For current version look in [REAME](README.md) 
## Step 2: Create the Main Function

Every Runa Engine app starts with a `main()` function that sets up the world and launches the application:

```rust
use runa_app::{RunaApp, RunaWindowConfig};
use runa_core::World;

fn main() {
    // Create a new world to hold game objects and systems
    let mut world = World::default();

    // Configure the application window
    let config = RunaWindowConfig {
        title: "My First Game".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: true,
        show_fps_in_title: true,
        window_icon: None,
    };

    // Launch the engine
    let _ = RunaApp::run_with_config(world, config);
}
```

## Step 3: Initialize Audio (Optional)

If you want to use audio in your game, initialize the audio engine:

```rust
let mut world = World::default();

// Initialize audio engine
world.audio_engine.initialize().expect("Failed to initialize audio");
```

## Step 4: Add Game Objects

Game objects are created by spawning scripts. Here's a simple example:

```rust
// Spawn a game object with a script
world.spawn(Box::new(MyObject::new()));
```

## Step 5: Run Your App

Build and run your project:

```bash
cargo run
```

## Window Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `title` | Window title | "Runa Game" |
| `width` | Window width in pixels | 1280 |
| `height` | Window height in pixels | 720 |
| `fullscreen` | Start in fullscreen mode | false |
| `vsync` | Enable vertical sync | true |
| `show_fps_in_title` | Show FPS in window title | false |

## Next Steps

- Learn about [Scripts](../scripts/creating-scripts.md) to add behavior to objects
- Explore [Components](../components/transform.md) to add properties to objects
- Check out the [Input](../systems/input.md) system for player controls

# Input System

The input system handles keyboard and mouse input for your game. In the current runtime it also exposes control over the **main runtime window**.

## Checking Keyboard Input

### Key States

```rust
use runa_core::input_system::*;
use winit::keyboard::KeyCode;

// Is key currently held down?
if Input::is_key_pressed(KeyCode::KeyW) {
    // Move forward
}

// Was key just pressed this frame? (triggers once)
if Input::is_key_just_pressed(KeyCode::Space) {
    // Jump!
}
```

### Common Keys

```rust
KeyCode::KeyW      // W key
KeyCode::KeyA      // A key
KeyCode::KeyS      // S key
KeyCode::KeyD      // D key
KeyCode::Space     // Spacebar
KeyCode::Enter     // Enter/Return
KeyCode::Escape    // Escape
KeyCode::KeyZ      // Z key
// ... and all other letter/number keys
```

## Mouse Input

### Mouse Position

```rust
use runa_core::input_system::*;

// Get mouse position in world coordinates
if let Some(mouse_pos) = Input::get_mouse_world_position() {
    println!("Mouse at: {:?}", mouse_pos);
}
```

### Mouse Buttons

```rust
use runa_core::input_system::*;
use winit::event::MouseButton;

// Is mouse button held?
if Input::is_mouse_button_pressed(MouseButton::Left) {
    // Left click held
}

// Was mouse button just clicked?
if Input::is_mouse_button_just_pressed(MouseButton::Left) {
    // Left click just happened
}

// Mouse wheel
let scroll = Input::current().mouse_wheel_delta;
```

## Window Control

The current runtime is single-window. These functions control the active main game window from scripts.

```rust
use runa_core::input_system::*;

set_window_title("Debug View");
set_fullscreen(true);
toggle_fullscreen();

set_window_size(1600, 900);
set_window_position(120, 80);
move_window_by(16, 0);
center_window();

let title = window_title();
let size = window_size();
let fullscreen = is_fullscreen();
let screen_center = screen_center_position();
let centered_window = centered_window_position();
```

## Complete Example: Player Movement

```rust
use runa_core::{
    components::Transform,
    input_system::*,
    ocs::{Object, Script},
    World,
    glam::Vec3,
};

pub struct Player {
    speed: f32,
}

impl Player {
    pub fn new() -> Self {
        Self { speed: 5.0 }
    }
}

impl Script for Player {
    fn update(&mut self, object: &mut Object, dt: f32) {
        if let Some(transform) = object.get_component_mut::<Transform>() {
            let mut direction = Vec3::ZERO;

            // WASD movement
            if Input::is_key_pressed(KeyCode::KeyW) {
                direction.y += 1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyS) {
                direction.y -= 1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyA) {
                direction.x -= 1.0;
            }
            if Input::is_key_pressed(KeyCode::KeyD) {
                direction.x += 1.0;
            }

            // Normalize for consistent diagonal speed
            if direction.length() > 0.0 {
                direction = direction.normalize();
            }

            // Apply movement
            transform.position += direction * self.speed * dt;
        }
    }
}
```

## Example: Click to Interact

```rust
use runa_core::{
    components::CursorInteractable,
    input_system::*,
    ocs::{Object, Script},
    World,
};

pub struct Button {
    clicked: bool,
}

impl Script for Button {
    fn construct(&self, object: &mut Object) {
        // Add interactable component
        let mut interactable = CursorInteractable::new(100.0, 50.0);

        // Set up callbacks
        interactable.set_on_hover_enter(|| {
            println!("Mouse entered button!");
        });

        interactable.set_on_hover_exit(|| {
            println!("Mouse exited button!");
        });

        object.add_component(interactable);
    }

    fn update(&mut self, object: &mut Object, _dt: f32) {
        if let Some(interactable) = object.get_component::<CursorInteractable>() {
            if interactable.is_hovered && Input::is_mouse_button_just_pressed(MouseButton::Left) {
                println!("Button clicked!");
            }
        }
    }
}
```

## Input State Reference

| Method                                     | Description                         |
| ------------------------------------------ | ----------------------------------- |
| `Input::is_key_pressed(key)`               | True while key is held              |
| `Input::is_key_just_pressed(key)`          | True for one frame when pressed     |
| `Input::is_mouse_button_pressed(btn)`      | True while button is held           |
| `Input::is_mouse_button_just_pressed(btn)` | True for one frame when clicked     |
| `Input::get_mouse_world_position()`        | Mouse position in world coordinates |
| `set_window_title(title)`                  | Change the main window title        |
| `set_fullscreen(bool)`                     | Enable/disable fullscreen           |
| `toggle_fullscreen()`                      | Toggle fullscreen                   |
| `set_window_size(w, h)`                    | Request a new window size           |
| `set_window_position(x, y)`                | Move window to an absolute position |
| `move_window_by(dx, dy)`                   | Move window by an offset            |
| `screen_center_position()`                 | Center of the current monitor       |
| `centered_window_position()`               | Centered position for the window    |
| `center_window()`                          | Move the window to monitor center   |

## Tips

- Use `is_key_just_pressed` for actions that should trigger once (jump, shoot)
- Use `is_key_pressed` for continuous actions (movement, charging)
- Always multiply movement by `dt` for frame-rate independence
- Mouse world position requires a camera to be set up
- Window-control functions affect the main runtime window only
- Centering helpers use the window's current monitor
- Multi-window runtime support is not implemented yet

## Next Steps

- [Transform](../components/transform.md) for moving objects
- [CursorInteractable](../components/cursor-interactable.md) for clickable objects
- [Creating a 2D Game](../getting-started/creating-a-2d-game.md) for camera setup

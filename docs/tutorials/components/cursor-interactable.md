# CursorInteractable Component

The `CursorInteractable` component makes objects respond to mouse hover and click events.

## Quick Start

```rust
use runa_core::components::CursorInteractable;

// Create an interactable with size (width, height)
let mut interactable = CursorInteractable::new(100.0, 50.0);

// Add callbacks
interactable.set_on_hover_enter(|| {
    println!("Mouse entered!");
});

interactable.set_on_hover_exit(|| {
    println!("Mouse exited!");
});

// Add to object
object.add_component(interactable);
```

## Setting Up Callbacks

### Hover Events

```rust
let mut interactable = CursorInteractable::new(100.0, 50.0);

// Called when mouse enters the bounds
interactable.set_on_hover_enter(|| {
    println!("Hover started!");
    // Change sprite color, play sound, etc.
});

// Called when mouse leaves the bounds
interactable.set_on_hover_exit(|| {
    println!("Hover ended!");
    // Reset sprite color, etc.
});
```

### Click Events

```rust
// Check for clicks in your script's update method
fn update(&mut self, object: &mut Object, _dt: f32, _world: &mut World) {
    if let Some(interactable) = object.get_component::<CursorInteractable>() {
        if interactable.is_hovered && Input::is_mouse_button_just_pressed(MouseButton::Left) {
            println!("Clicked!");
            // Handle click
        }
    }
}
```

## Complete Example: Button

```rust
use runa_core::{
    components::{CursorInteractable, SpriteRenderer, Transform},
    input_system::*,
    ocs::{Object, Script},
    World,
    glam::Vec3,
};

pub struct Button {
    is_hovered: bool,
}

impl Button {
    pub fn new() -> Self {
        Self { is_hovered: false }
    }
}

impl Script for Button {
    fn construct(&self, object: &mut Object) {
        // Create interactable
        let mut interactable = CursorInteractable::new(200.0, 60.0);
        
        interactable.set_on_hover_enter(|| {
            println!("Button hover!");
        });
        
        interactable.set_on_hover_exit(|| {
            println!("Button unhover!");
        });
        
        object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::load_image!("assets/button.png")),
            })
            .add_component(interactable);
    }

    fn start(&mut self, object: &mut Object) {
        if let Some(transform) = object.get_component_mut::<Transform>() {
            transform.position = Vec3::new(0.0, 0.0, 0.0);
        }
    }

    fn update(&mut self, object: &mut Object, _dt: f32, _world: &mut World) {
        if let Some(interactable) = object.get_component::<CursorInteractable>() {
            if interactable.is_hovered && Input::is_mouse_button_just_pressed(MouseButton::Left) {
                println!("Button clicked!");
                // Handle button click
            }
        }
    }
}
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `is_hovered` | `bool` | Is mouse currently over the object |
| `is_pressed` | `bool` | Is mouse button pressed while hovering |
| `was_hovered` | `bool` | Was hovered last frame (for detecting transitions) |
| `bounds_size` | `Vec3` | Half-size of the interaction area |

## Methods

```rust
// Create 2D interactable (width, height)
let interactable = CursorInteractable::new(100.0, 50.0);

// Create 3D interactable (width, height, depth)
let interactable = CursorInteractable::new_3d(100.0, 50.0, 10.0);

// Check if point is inside bounds
let point = Vec3::new(10.0, 20.0, 0.0);
let center = Vec3::new(0.0, 0.0, 0.0);
if interactable.contains_point(point, center) {
    println!("Point is inside!");
}
```

## Tips

- The bounds size is **half** the total size (extents)
- For a 100x50 button, use `new(50.0, 25.0)`
- Callbacks are automatically called when hover state changes
- Use `is_mouse_button_just_pressed` for single-click detection

## Next Steps

- [Input](../systems/input.md) for more mouse controls
- [SpriteRenderer](sprite-renderer.md) for button visuals
- [Audio](../systems/audio.md) for click sounds

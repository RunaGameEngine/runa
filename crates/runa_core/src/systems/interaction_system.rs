use crate::{
    components::{CursorInteractable, Transform},
    input_system::*,
    ocs::World,
};
use glam::Vec3;

pub struct InteractionSystem {
    mouse_just_pressed: bool,
    mouse_position: Vec3,
    mouse_just_released: bool,
    pressed_object_index: Option<usize>, // Track which object was pressed
}

impl InteractionSystem {
    pub fn new() -> Self {
        Self {
            mouse_position: Vec3::ZERO,
            mouse_just_pressed: false,
            mouse_just_released: false,
            pressed_object_index: None,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        self.mouse_position = Input::get_mouse_world_position().unwrap_or(Vec3::ZERO);

        // Reset states
        for object in &mut world.objects {
            if let Some(interactable) = object.get_component_mut::<CursorInteractable>() {
                interactable.is_hovered = false;
                interactable.is_pressed = false;
            }
        }

        // Find the object under cursor
        let mut min_distance = f32::MAX;
        let mut closest_object_idx = None;

        // First pass: find which object is closest to the cursor
        for (index, object) in world.objects.iter().enumerate() {
            if let (Some(transform), Some(interactable)) = (
                object.get_component::<Transform>(),
                object.get_component::<CursorInteractable>(),
            ) {
                if interactable.contains_point(self.mouse_position, transform.position) {
                    let distance = self.mouse_position.distance_squared(transform.position);
                    if distance < min_distance {
                        min_distance = distance;
                        closest_object_idx = Some(index);
                    }
                }
            }
        }

        // Second pass: update hover states
        for (index, object) in world.objects.iter_mut().enumerate() {
            if let Some(interactable) = object.get_component_mut::<CursorInteractable>() {
                if Some(index) == closest_object_idx {
                    interactable.is_hovered = true;

                    // If mouse is just pressed while hovering this object, store its index
                    if self.mouse_just_pressed {
                        self.pressed_object_index = Some(index);
                        interactable.is_pressed = true;
                    }
                } else {
                    interactable.is_hovered = false;
                }
            }
        }

        // Handle click when mouse is released over the same object that was pressed
        if self.mouse_just_released {
            // Only trigger click if the mouse was pressed and released over the same object
            if let (Some(pressed_idx), Some(closest_idx)) =
                (self.pressed_object_index, closest_object_idx)
            {
                if pressed_idx == closest_idx {
                    if let Some(object) = world.objects.get_mut(closest_idx) {
                        if let Some(interactable) = object.get_component_mut::<CursorInteractable>()
                        {
                            if let Some(ref mut callback) = interactable.on_click {
                                callback();
                            }
                        }
                    }
                }
            }

            // Reset the pressed object index after release
            self.pressed_object_index = None;
        }

        // Update callbacks
        for object in &mut world.objects {
            if let Some(interactable) = object.get_component_mut::<CursorInteractable>() {
                interactable.update_callbacks();
            }
        }
    }
}

use crate::{
    components::{CursorInteractable, Transform},
    input::InputState,
    ocs::{ObjectId, World},
};
use glam::Vec3;

pub struct InteractionSystem {
    mouse_just_pressed: bool,
    mouse_position: Vec3,
    mouse_just_released: bool,
    pressed_object_id: Option<ObjectId>,
}

impl InteractionSystem {
    pub fn new() -> Self {
        Self {
            mouse_position: Vec3::ZERO,
            mouse_just_pressed: false,
            mouse_just_released: false,
            pressed_object_id: None,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        self.mouse_position = InputState::get_mouse_world_position().unwrap_or(Vec3::ZERO);
        let interactable_ids = world.find_all_with::<CursorInteractable>();

        // Reset states
        for object_id in &interactable_ids {
            let Some(object) = world.object_mut(*object_id) else {
                continue;
            };
            if let Some(interactable) = object.get_component_mut::<CursorInteractable>() {
                interactable.is_hovered = false;
                interactable.is_pressed = false;
            }
        }

        // Find the object under cursor
        let mut min_distance = f32::MAX;
        let mut closest_object_id = None;

        // First pass: find which object is closest to the cursor
        for object_id in &interactable_ids {
            let Some(object) = world.object(*object_id) else {
                continue;
            };
            if let (Some(transform), Some(interactable)) = (
                object.get_component::<Transform>(),
                object.get_component::<CursorInteractable>(),
            ) {
                if interactable.contains_point(self.mouse_position, transform.position) {
                    let distance = self.mouse_position.distance_squared(transform.position);
                    if distance < min_distance {
                        min_distance = distance;
                        closest_object_id = Some(*object_id);
                    }
                }
            }
        }

        // Second pass: update hover states
        for object_id in &interactable_ids {
            let Some(object) = world.object_mut(*object_id) else {
                continue;
            };
            if let Some(interactable) = object.get_component_mut::<CursorInteractable>() {
                if Some(*object_id) == closest_object_id {
                    interactable.is_hovered = true;

                    // If mouse is just pressed while hovering this object, store its id
                    if self.mouse_just_pressed {
                        self.pressed_object_id = Some(*object_id);
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
            if let (Some(pressed_id), Some(closest_id)) =
                (self.pressed_object_id, closest_object_id)
            {
                if pressed_id == closest_id {
                    if let Some(object) = world.object_mut(closest_id) {
                        if let Some(interactable) = object.get_component_mut::<CursorInteractable>()
                        {
                            if let Some(ref mut callback) = interactable.on_click_mut() {
                                if let Ok(mut cb) = callback.lock() {
                                    cb();
                                }
                            }
                        }
                    }
                }
            }

            // Reset the pressed object id after release
            self.pressed_object_id = None;
        }

        // Update callbacks
        for object_id in interactable_ids {
            let Some(object) = world.object_mut(object_id) else {
                continue;
            };
            if let Some(interactable) = object.get_component_mut::<CursorInteractable>() {
                interactable.update_callbacks();
            }
        }
    }
}

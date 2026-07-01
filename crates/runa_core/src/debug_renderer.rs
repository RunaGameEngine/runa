use runa_render_api::RenderQueue;

use crate::{
    components::{PhysicsCollision, Transform},
    ocs::World,
};

#[derive(Default)]
pub struct DebugRenderer {
    debug_draw_collisions: bool,
}

impl DebugRenderer {
    pub fn new() -> Self {
        Self {
            debug_draw_collisions: false,
        }
    }

    pub fn set_debug_draw_collisions(&mut self, enabled: bool) {
        self.debug_draw_collisions = enabled;
    }

    pub fn is_debug_draw_collisions_enabled(&self) -> bool {
        self.debug_draw_collisions
    }

    pub fn render_debug(&self, world: &World, _render_queue: &mut RenderQueue) {
        if !self.debug_draw_collisions {
            return;
        }

        for object_id in world.find_all_with::<PhysicsCollision>() {
            let Some(object) = world.object(object_id) else {
                continue;
            };
            if let (Some(_transform), Some(_collision)) = (
                object.get_component::<Transform>(),
                object.get_component::<PhysicsCollision>(),
            ) {
                // The current RenderQueue has no draw_line support.
                // Collision visualization can later be represented with
                // a small center sprite or a dedicated debug primitive pass.
            }
        }
    }
}

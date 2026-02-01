use runa_render_api::queue::RenderQueue;

use crate::{
    components::{physics_collision::PhysicsCollision, transform::Transform},
    ocs::world::World,
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

    pub fn render_debug(&self, world: &World, render_queue: &mut RenderQueue) {
        if !self.debug_draw_collisions {
            return;
        }

        for object in &world.objects {
            if let (Some(transform), Some(collision)) = (
                object.get_component::<Transform>(),
                object.get_component::<PhysicsCollision>(),
            ) {
                // В текущей реализации RenderQueue нет draw_line,
                // поэтому отображаем коллизию как маленький спрайт в центре
                // или добавим возможность отрисовки в будущем
            }
        }
    }
}

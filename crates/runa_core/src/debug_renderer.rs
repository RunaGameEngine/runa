use glam::Vec2;
use runa_render_api::RenderQueue;

use crate::{
    components::{Camera, CursorInteractable, PhysicsCollision, Transform},
    ocs::World,
};

#[derive(Default)]
pub struct DebugRenderer {
    debug_draw_collisions: bool,
    pub debug_show_cursor_bounds: bool,
}

impl DebugRenderer {
    pub fn new() -> Self {
        Self {
            debug_draw_collisions: false,
            debug_show_cursor_bounds: false,
        }
    }

    pub fn set_debug_draw_collisions(&mut self, enabled: bool) {
        self.debug_draw_collisions = enabled;
    }

    pub fn is_debug_draw_collisions_enabled(&self) -> bool {
        self.debug_draw_collisions
    }

    pub fn render_debug(&self, world: &World, render_queue: &mut RenderQueue) {
        let active_camera = world.objects.iter().find_map(|obj| {
            let cam = obj.get_component::<Camera>()?;
            obj.get_component::<crate::components::ActiveCamera>()?;
            Some(cam.resolved_with_transform(obj.get_component::<Transform>()))
        });

        if self.debug_draw_collisions {
            for object_id in world.find_all_with::<PhysicsCollision>() {
                let Some(object) = world.object(object_id) else {
                    continue;
                };
                if let (Some(transform), Some(collision)) = (
                    object.get_component::<Transform>(),
                    object.get_component::<PhysicsCollision>(),
                ) {
                    draw_world_aabb_outline(
                        transform.position.truncate(),
                        collision.size,
                        [1.0, 0.5, 0.0, 0.8],
                        &active_camera,
                        render_queue,
                    );
                }
            }
        }

        if self.debug_show_cursor_bounds {
            for object_id in world.find_all_with::<CursorInteractable>() {
                let Some(object) = world.object(object_id) else {
                    continue;
                };
                if let (Some(transform), Some(interactable)) = (
                    object.get_component::<Transform>(),
                    object.get_component::<CursorInteractable>(),
                ) {
                    let half = interactable.bounds_size.truncate();
                    draw_world_aabb_outline(
                        transform.position.truncate(),
                        half,
                        [0.0, 0.8, 1.0, 0.8],
                        &active_camera,
                        render_queue,
                    );
                }
            }
        }
    }
}

fn world_point_to_screen(world_pos: Vec2, camera: &Camera) -> Vec2 {
    let visible = camera.ortho_visible_size();
    let half_w = visible.x * 0.5;
    let half_h = visible.y * 0.5;
    let ndc_x = (world_pos.x - camera.position.x) / half_w;
    let ndc_y = (world_pos.y - camera.position.y) / half_h;
    Vec2::new(
        (ndc_x + 1.0) * 0.5 * camera.viewport_size.0 as f32,
        (1.0 - ndc_y) * 0.5 * camera.viewport_size.1 as f32,
    )
}

fn draw_world_aabb_outline(
    center: Vec2,
    half_extents: Vec2,
    color: [f32; 4],
    camera: &Option<Camera>,
    render_queue: &mut RenderQueue,
) {
    let Some(cam) = camera else { return };
    let corners = [
        center + Vec2::new(-half_extents.x, -half_extents.y),
        center + Vec2::new(half_extents.x, -half_extents.y),
        center + Vec2::new(half_extents.x, half_extents.y),
        center + Vec2::new(-half_extents.x, half_extents.y),
    ];
    let screen: Vec<Vec2> = corners.iter().map(|&p| world_point_to_screen(p, cam)).collect();
    for i in 0..4 {
        let j = (i + 1) % 4;
        render_queue.draw_debug_line(screen[i], screen[j], color, 1.5);
    }
}

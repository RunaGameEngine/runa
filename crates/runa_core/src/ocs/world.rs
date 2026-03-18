use std::sync::Arc;

use glam::Vec3;
use runa_render_api::RenderQueue;

use crate::{
    components::{SpriteRenderer, Tilemap, Transform},
    debug_renderer::DebugRenderer,
    ocs::{Object, Script},
};

#[derive(Default)]
pub struct World {
    pub objects: Vec<Object>,
    debug_renderer: DebugRenderer,
}

impl World {
    pub fn default() -> Self {
        Self {
            objects: Vec::new(),
            debug_renderer: DebugRenderer::new(),
        }
    }

    pub fn spawn(&mut self, script: Box<dyn Script>) -> &Object {
        let mut object = Object::new();
        object.set_script(script);

        self.objects.push(object);
        self.objects.get(self.objects.len() - 1).unwrap()
    }

    pub fn construct(&mut self) {
        for object in &mut self.objects {
            if let Some(script) = object.script.take() {
                script.construct(object);
                object.script = Some(script);
            }
        }
    }

    pub fn start(&mut self) {
        for object in &mut self.objects {
            if let Some(mut script) = object.script.take() {
                script.start(object);
                object.script = Some(script);
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        for object in &mut self.objects {
            if let Some(transform) = object.get_component_mut::<Transform>() {
                transform.prepare_for_update();
            }
        }

        for object in &mut self.objects {
            if let Some(mut script) = object.script.take() {
                script.update(object, dt);
                object.script = Some(script);
            }
        }
    }

    pub fn render(&self, render_queue: &mut RenderQueue, interpolation_factor: f32) {
        for object in &self.objects {
            if let (Some(transform), Some(sprite)) = (
                object.get_component::<Transform>(),
                object.get_component::<SpriteRenderer>(),
            ) {
                // Интерполируем позицию
                let interpolated_position = Vec3::lerp(
                    transform.previous_position,
                    transform.position,
                    interpolation_factor,
                );

                let interpolated_rotation = transform.previous_rotation
                    + (transform.rotation - transform.previous_rotation) * interpolation_factor;

                render_queue.push_sprite(
                    Arc::from(sprite.get_texture_handle()),
                    interpolated_position,
                    interpolated_rotation,
                    transform.scale,
                );
            }
            if let (Some(tilemap), Some(transform)) = (
                object.get_component::<Tilemap>(),
                object.get_component::<Transform>(),
            ) {
                for layer in &tilemap.layers {
                    if !layer.visible {
                        continue;
                    }

                    for y in tilemap.offset.y..(tilemap.offset.y + tilemap.height as i32) {
                        for x in tilemap.offset.x..(tilemap.offset.x + tilemap.width as i32) {
                            let array_x = (x - tilemap.offset.x) as u32;
                            let array_y = (y - tilemap.offset.y) as u32;

                            if let Some(tile) = layer.get_tile(array_x as u32, array_y as u32) {
                                if tile.texture.is_none() {
                                    continue;
                                }

                                // Мировая позиция тайла относительно объекта
                                let world_pos = tilemap.tile_to_world(x, y);
                                let final_pos = transform.position + world_pos;

                                render_queue.push_tile(
                                    tile.texture.clone().unwrap(),
                                    final_pos,
                                    tilemap.tile_size,
                                    [
                                        tile.uv_rect.x,
                                        tile.uv_rect.y,
                                        tile.uv_rect.width,
                                        tile.uv_rect.height,
                                    ],
                                    tile.flip_x,
                                    tile.flip_y,
                                    [1.0, 1.0, 1.0, 1.0],
                                );
                            }
                        }
                    }
                }
            }
        }

        // Отладочная отрисовка
        self.debug_renderer.render_debug(self, render_queue);
    }

    pub fn set_debug_draw_collisions(&mut self, enabled: bool) {
        self.debug_renderer.set_debug_draw_collisions(enabled);
    }

    pub fn is_debug_draw_collisions_enabled(&self) -> bool {
        self.debug_renderer.is_debug_draw_collisions_enabled()
    }

    pub fn objects_mut(&mut self) -> &mut Vec<Object> {
        &mut self.objects
    }
}

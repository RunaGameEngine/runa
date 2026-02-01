use glam::Vec2;
use runa_render_api::queue::RenderQueue;

use crate::{
    components::{sprite_renderer::SpriteRenderer, transform::Transform},
    debug_renderer::DebugRenderer,
    input::InputState,
    ocs::{object::Object, script::Script},
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
    pub fn input(&mut self, input: &InputState) {
        for object in &mut self.objects {
            if let Some(mut script) = object.script.take() {
                script.input(object, input);
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
                let interpolated_position = Vec2::lerp(
                    transform.previous_position,
                    transform.position,
                    interpolation_factor,
                );

                let interpolated_rotation = transform.previous_rotation
                    + (transform.rotation - transform.previous_rotation) * interpolation_factor;

                render_queue.draw_sprite(
                    sprite.get_texture_handle(),
                    interpolated_position,
                    interpolated_rotation,
                    transform.scale,
                );
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
}

use std::sync::Arc;

use glam::Vec3;
use runa_render_api::RenderQueue;

use crate::{
    audio::{AudioEngine, SoundId},
    components::{
        ActiveCamera, AudioListener, AudioSource, Camera, Canvas, MeshRenderer, SpriteRenderer,
        Tilemap, Transform,
    },
    debug_renderer::DebugRenderer,
    ocs::{Object, Script},
};

#[derive(Default)]
pub struct World {
    pub objects: Vec<Object>,
    debug_renderer: DebugRenderer,
    pub audio_engine: AudioEngine,
}

impl World {
    /// Play a sound through the audio engine
    pub fn play_sound(&mut self, audio_source: &AudioSource) -> Option<SoundId> {
        self.audio_engine.play(audio_source)
    }

    pub fn default() -> Self {
        Self {
            objects: Vec::new(),
            debug_renderer: DebugRenderer::new(),
            audio_engine: AudioEngine::default(),
        }
    }

    pub fn spawn(&mut self, script: Box<dyn Script>) -> &Object {
        let mut object = Object::new();
        object.set_script(script);

        self.objects.push(object);

        // Set world pointer using raw pointer (safe because world outlives objects)
        let world_ptr = self as *mut World;
        let object = self.objects.last_mut().unwrap();
        object.set_world(unsafe { &mut *world_ptr });

        object
    }

    pub fn construct(&mut self) {
        self.audio_engine
            .initialize()
            .expect("Failed to initialize audio engine");

        // Set world pointers for all objects using raw pointer (safe because world outlives objects)
        let world_ptr = self as *mut World;
        for object in &mut self.objects {
            object.set_world(unsafe { &mut *world_ptr });

            if let Some(script) = object.script.take() {
                script.construct(object);
                object.script = Some(script);
            }
        }
    }

    pub fn start(&mut self) {
        // Set world pointers for all objects using raw pointer (safe because world outlives objects)
        let world_ptr = self as *mut World;
        for object in &mut self.objects {
            object.set_world(unsafe { &mut *world_ptr });

            if let Some(mut script) = object.script.take() {
                script.start(object);
                object.script = Some(script);
            }

            // Handle play_on_awake for AudioSource components
            if let Some(audio) = object.get_component_mut::<AudioSource>() {
                if audio.play_on_awake && audio.audio_asset.is_some() {
                    audio.play_requested = true;
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        for object in &mut self.objects {
            if let Some(transform) = object.get_component_mut::<Transform>() {
                transform.prepare_for_update();
            }
        }

        // Update scripts
        let object_count = self.objects.len();
        for i in 0..object_count {
            if let Some(mut script) = self.objects[i].script.take() {
                script.update(&mut self.objects[i], dt);
                self.objects[i].script = Some(script);
            }
        }

        // Find active AudioListener and update listener position
        let mut listener_found = false;
        for object in &self.objects {
            if let (Some(listener), Some(transform)) = (
                object.get_component::<AudioListener>(),
                object.get_component::<Transform>(),
            ) {
                if listener.active {
                    self.audio_engine.set_listener(
                        transform.position,
                        transform.rotation,
                        listener.volume,
                    );
                    self.audio_engine
                        .set_stereo_separation(listener.stereo_separation);
                    listener_found = true;
                    break;
                }
            }
        }

        // If no active listener found, use default position
        if !listener_found {
            self.audio_engine
                .set_listener(Vec3::ZERO, glam::Quat::IDENTITY, 1.0);
        }

        // Update spatial sound volumes based on listener position
        self.audio_engine.update_spatial_volumes();

        // Process audio requests (play/stop) from AudioSource components
        for object in &mut self.objects {
            // Get sound position for 3D audio first (to avoid borrow conflicts)
            let sound_position = object.get_component::<Transform>().map(|t| t.position);

            if let Some(audio) = object.get_component_mut::<AudioSource>() {
                // Handle stop requests first
                if audio.stop_requested {
                    if let Some(sound_id) = audio.sound_id.take() {
                        self.audio_engine.stop(sound_id);
                    }
                    audio.playing = false;
                    audio.stop_requested = false;
                }

                // Handle play requests
                if audio.play_requested && audio.audio_asset.is_some() {
                    // Stop previous sound if still playing
                    if let Some(sound_id) = audio.sound_id.take() {
                        self.audio_engine.stop(sound_id);
                    }

                    // Play new sound with spatial positioning if needed
                    let sound_id = self.audio_engine.play_spatial(audio, sound_position);

                    if let Some(id) = sound_id {
                        audio.sound_id = Some(id);
                        audio.playing = true;
                    }
                    audio.play_requested = false;
                }
            }
        }

        // Cleanup finished sounds
        self.audio_engine.cleanup();

        // UI
        for object in &mut self.objects {
            let viewport_size = if let (Some(camera), Some(_active)) = (
                object.get_component::<Camera>(),
                object.get_component::<ActiveCamera>(),
            ) {
                Some(glam::Vec2::new(
                    camera.viewport_size.0 as f32,
                    camera.viewport_size.1 as f32,
                ))
            } else {
                None
            };

            if let (Some(canvas), Some(viewport_size)) =
                (object.get_component_mut::<Canvas>(), viewport_size)
            {
                if canvas.dirty_layout {
                    canvas.layout(viewport_size);
                }
            }
        }
    }

    pub fn render(&self, render_queue: &mut RenderQueue, interpolation_factor: f32) {
        for object in &self.objects {
            // 3D Mesh rendering
            if let (Some(transform), Some(mesh_renderer)) = (
                object.get_component::<Transform>(),
                object.get_component::<MeshRenderer>(),
            ) {
                // Convert Mesh vertices to render_api Vertex3D
                let vertices: Vec<runa_render_api::command::Vertex3D> = mesh_renderer
                    .mesh
                    .vertices
                    .iter()
                    .map(|v| runa_render_api::command::Vertex3D {
                        position: v.position,
                        normal: v.normal,
                        uv: v.uv,
                    })
                    .collect();

                // Create model matrix
                let model_matrix = glam::Mat4::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.position,
                );

                render_queue.draw_mesh_3d(
                    vertices,
                    mesh_renderer.mesh.indices.clone(),
                    model_matrix,
                    mesh_renderer.color,
                );
            }

            // 2D Sprite rendering
            if let (Some(transform), Some(sprite)) = (
                object.get_component::<Transform>(),
                object.get_component::<SpriteRenderer>(),
            ) {
                let Some(texture) = sprite.texture.clone() else {
                    continue;
                };

                // Interpolate position
                let interpolated_position = Vec3::lerp(
                    transform.previous_position,
                    transform.position,
                    interpolation_factor,
                );

                let interpolated_rotation = transform.previous_rotation
                    + (transform.rotation - transform.previous_rotation) * interpolation_factor;

                render_queue.draw_sprite(
                    Arc::from(texture),
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

                                // Tile world position relative to the object
                                let world_pos = tilemap.tile_to_world(x, y);
                                let final_pos = transform.position + world_pos;

                                render_queue.draw_tile(
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

            if let (Some(canvas), Some(_camera), Some(_ac)) = (
                &mut object.get_component::<Canvas>(),
                object.get_component::<Camera>(),
                object.get_component::<ActiveCamera>(),
            ) {
                canvas.build_render_commands(render_queue);
            }
        }

        // Debug rendering
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

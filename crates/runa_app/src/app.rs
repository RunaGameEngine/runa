use std::sync::Arc;
use std::time::{Duration, Instant};

use runa_core::components::{Camera, MeshRenderer, SpriteRenderer, Transform};
use runa_core::input::InputState;
use runa_core::{glam, Console};
use runa_ecs::R;
use runa_render::Renderer;
use runa_render_api::{Mesh3dParams, RenderQueue};

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Debug, Clone)]
pub struct RunaWindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub show_fps_in_title: bool,
    pub window_icon: Option<String>,
}

impl Default for RunaWindowConfig {
    fn default() -> Self {
        Self {
            title: "Runa Game".to_string(),
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: true,
            show_fps_in_title: false,
            window_icon: None,
        }
    }
}

pub struct App<'window> {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer<'window>>,

    pub queue: RenderQueue,
    pub ecs_world: runa_ecs::World,
    pub scheduler: runa_ecs::Scheduler,

    // Timing
    pub last_time: Instant,
    pub accumulator: f32,
    pub frame_count: u32,
    pub current_fps: f32,
    pub last_fps_update: Instant,
    pub last_frame_time: f32,
    pub current_frame_time_ms: f32,
    pub current_render_time_ms: f32,
    pub current_update_time_ms: f32,

    pub console: Console,

    pub config: RunaWindowConfig,
    pub frame_start: Instant,
}

impl<'window> App<'window> {
    fn toggle_fullscreen(&mut self) {
        runa_core::input::toggle_fullscreen();
        self.config.fullscreen = runa_core::input::is_fullscreen().unwrap_or(false);
    }

    fn sync_camera(&mut self) {
        if let Some(renderer) = &self.renderer {
            let w = renderer.surface_config.width;
            let h = renderer.surface_config.height;
            for (_, cam) in self.ecs_world.query_mut::<runa_ecs::W<Camera>>() {
                cam.resize(w, h);
            }
        }
    }

    fn render_ecs_sprites(&mut self) {
        let Self { ref ecs_world, ref mut queue, .. } = self;
        for (_, (transform, sprite)) in ecs_world.query::<(R<Transform>, R<SpriteRenderer>)>() {
            if let Some(tex) = sprite.texture() {
                queue.draw_sprite(
                    tex.inner.clone(),
                    transform.position,
                    transform.rotation,
                    transform.scale,
                    [1.0; 4],
                    sprite.uv_rect,
                    0,
                );
            }
        }
    }

    fn render_ecs_meshes(&mut self) {
        let Self { ref ecs_world, ref mut queue, .. } = self;
        for (_, (transform, renderer)) in ecs_world.query::<(R<Transform>, R<MeshRenderer>)>() {
            let Some(handle) = &renderer.mesh else { continue };
            let mesh = &handle.inner;
            let model = glam::Mat4::from_scale_rotation_translation(
                transform.scale,
                transform.rotation,
                transform.position,
            );
            let mesh_id = mesh.vertices.as_ptr() as u64;
            let vtx: Vec<runa_render_api::Vertex3D> = mesh.vertices.iter().map(|v| runa_render_api::Vertex3D {
                position: v.position,
                normal: v.normal,
                uv: v.uv,
                color: v.color,
            }).collect();
            queue.draw_mesh_3d(Mesh3dParams {
                mesh_id,
                vertices: vtx,
                indices: mesh.indices.clone(),
                model_matrix: model,
                color: renderer.color,
                emission: [0.0; 3],
                use_vertex_color: true,
                order: 0,
                depth: transform.position.z,
            });
        }
    }

    fn render(&mut self) {
        let render_start = Instant::now();

        let camera = self
            .ecs_world
            .query::<runa_ecs::R<Camera>>()
            .next()
            .map(|(_, c)| *c)
            .unwrap_or_default();

        // Phase 1: populate queue from ECS (no renderer borrow)
        self.queue.clear();
        self.render_ecs_sprites();
        self.render_ecs_meshes();

        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            self.console.current_fps = self.current_fps;
            self.console.current_frame_time_ms = self.current_frame_time_ms;
            self.console.current_render_time_ms = self.current_render_time_ms;
            self.console.current_update_time_ms = self.current_update_time_ms;
            self.console.draw_call_count = self.queue.commands.len();
            self.console.render(&mut self.queue, &camera);

            let camera_matrix = camera.matrix();
            let virtual_size = if matches!(
                camera.projection,
                runa_core::components::ProjectionType::Perspective
            ) {
                glam::Vec2::new(
                    renderer.surface_config.width.max(1) as f32,
                    renderer.surface_config.height.max(1) as f32,
                )
            } else {
                camera.orthographic_size
            };

            renderer.draw(&self.queue, camera_matrix, virtual_size);

            self.current_render_time_ms = render_start.elapsed().as_secs_f32() * 1000.0;

            self.frame_count += 1;
            let now = Instant::now();
            if now.duration_since(self.last_fps_update).as_secs_f32() >= 1.0 {
                self.current_fps = self.frame_count as f32
                    / now.duration_since(self.last_fps_update).as_secs_f32();
                self.frame_count = 0;
                self.last_fps_update = now;
                self.config.title = runa_core::input::window_title()
                    .unwrap_or_else(|| self.config.title.clone());
                if self.config.show_fps_in_title {
                    window.set_title(&format!(
                        "{} - {:.1} FPS",
                        self.config.title, self.current_fps
                    ));
                } else {
                    window.set_title(&format!("{}", self.config.title));
                }
            }
        }
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_title(&format!("{}", self.config.title))
                .with_visible(false);
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );

            if let Some(icon_path) = &self.config.window_icon {
                match runa_asset::load_window_icon(icon_path) {
                    Ok(icon) => {
                        window.set_window_icon(Some(icon));
                        println!("Window icon loaded: {}", icon_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to load window icon '{}': {}", icon_path, e);
                    }
                }
            } else {
                match runa_asset::load_window_icon(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/icon.png"
                )) {
                    Ok(icon) => {
                        window.set_window_icon(Some(icon));
                    }
                    Err(_) => {}
                }
            }

            runa_core::input::initialize_window_state(
                self.config.title.clone(),
                self.config.fullscreen,
                (self.config.width, self.config.height),
            );
            self.window = Some(window.clone());

            runa_core::input::set_window_handle(&window);
            runa_core::input::set_window_size(self.config.width, self.config.height);
            runa_core::input::set_fullscreen(self.config.fullscreen);

            let renderer = Renderer::new(window.clone(), self.config.vsync);
            self.renderer = Some(renderer);
            self.sync_camera();
            window.request_redraw();
            window.set_visible(true);
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        self.frame_start = Instant::now();

        let current_time = self.frame_start;
        let frame_time = (current_time - self.last_time).as_secs_f32().min(0.1);
        self.last_frame_time = frame_time;
        self.current_frame_time_ms = frame_time * 1000.0;
        self.last_time = current_time;

        let base_timestep = 1.0 / 60.0;
        let scaled_timestep = base_timestep / self.console.time_scale.max(0.01);

        self.accumulator += frame_time;

        let update_start = Instant::now();

        while self.accumulator >= scaled_timestep {
            {
                let mut input_state = InputState::current_mut();
                input_state.camera = self
                    .ecs_world
                    .query::<runa_ecs::R<Camera>>()
                    .next()
                    .map(|(_, c)| *c);
            }

            self.scheduler.run(&mut self.ecs_world);

            InputState::update_frame();

            self.accumulator -= scaled_timestep;
        }

        self.current_update_time_ms = update_start.elapsed().as_secs_f32() * 1000.0;

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                let had_window = self.window.is_some();
                if let Some(wgpu_ctx) = self.renderer.as_mut() {
                    wgpu_ctx.resize((new_size.width, new_size.height));
                    for (_, cam) in self.ecs_world.query_mut::<runa_ecs::W<Camera>>() {
                        cam.resize(new_size.width, new_size.height);
                    }
                    self.config.width = new_size.width;
                    self.config.height = new_size.height;
                    runa_core::input::initialize_window_state(
                        runa_core::input::window_title()
                            .unwrap_or_else(|| self.config.title.clone()),
                        runa_core::input::is_fullscreen().unwrap_or(self.config.fullscreen),
                        (new_size.width, new_size.height),
                    );
                }
                if had_window {
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();

                let fps_max = self.console.fps_max;

                if fps_max.is_finite() && fps_max > 0.0 {
                    let min_frame_time = Duration::from_secs_f32(1.0 / fps_max);
                    let elapsed = self.frame_start.elapsed();

                    if elapsed < min_frame_time {
                        let remaining = min_frame_time - elapsed;
                        if remaining > Duration::from_millis(1) {
                            std::thread::sleep(remaining - Duration::from_millis(1));
                        }
                        while self.frame_start.elapsed() < min_frame_time {}
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F11),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                self.toggle_fullscreen();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.console.handle_keyboard(&event, event.state);

                if !self.console.is_visible() {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        let mut input_state = InputState::current_mut();
                        if event.state == ElementState::Pressed {
                            input_state.keys_pressed.insert(key_code);
                            input_state.keys_just_pressed.insert(key_code);
                        } else {
                            input_state.keys_pressed.remove(&key_code);
                            input_state.keys_just_pressed.remove(&key_code);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let mut input_state = InputState::current_mut();
                input_state.mouse_position = (position.x as f32, position.y as f32);
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    let mut input_state = InputState::current_mut();
                    input_state.mouse_wheel_delta = y;
                }
                _ => {}
            },

            WindowEvent::MouseInput { state, button, .. } => {
                let mut input_state = InputState::current_mut();
                if state == ElementState::Pressed {
                    input_state.mouse_buttons_pressed.insert(button);
                    input_state.mouse_buttons_just_pressed.insert(button);
                } else {
                    input_state.mouse_buttons_pressed.remove(&button);
                    input_state.mouse_buttons_just_released.insert(button);
                }
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let mut input_state = InputState::current_mut();
            input_state.mouse_delta.0 += delta.0 as f32;
            input_state.mouse_delta.1 += delta.1 as f32;
        }
    }
}

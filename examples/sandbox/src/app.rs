use std::sync::Arc;
use std::time::Instant;

use runa_core::components::camera2d::Camera2D;
use runa_core::console::Console;
use runa_core::input::InputState;
use runa_core::ocs::world::World;
use runa_core::systems::interaction_system::InteractionSystem;
use runa_render::renderer::Renderer;
use runa_render_api::queue::RenderQueue;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

pub struct App<'window> {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer<'window>>,

    pub queue: RenderQueue,
    pub camera: Camera2D,
    pub world: World,

    pub last_time: Instant,
    pub accumulator: f32,
    pub frame_count: u32,
    pub last_fps_update: Instant,

    pub is_fullscreen: bool,
    pub input_state: InputState,
    pub interaction_system: InteractionSystem,

    pub console: Console,
}

impl<'window> App<'window> {
    fn toggle_fullscreen(&mut self) {
        if let Some(window) = &self.window {
            self.is_fullscreen = !self.is_fullscreen;

            if self.is_fullscreen {
                // Вход в полноэкранный режим
                let fullscreen = Some(Fullscreen::Borderless(window.current_monitor()));
                window.set_fullscreen(fullscreen);
            } else {
                // Выход из полноэкранного режима
                window.set_fullscreen(None);
            }
        }
    }

    fn render(&mut self, interpolation_factor: f32) {
        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            // Очищаем очередь
            self.queue.clear();

            // Собираем команды
            self.world.render(&mut self.queue, interpolation_factor);

            // Рендерим
            renderer.draw(&self.queue, self.camera.matrix(), self.camera.virtual_size);

            // Обновляем FPS
            self.frame_count += 1;
            let now = Instant::now();
            if now.duration_since(self.last_fps_update).as_secs_f32() >= 1.0 {
                let fps = self.frame_count as f32
                    / now.duration_since(self.last_fps_update).as_secs_f32();
                self.frame_count = 0;
                self.last_fps_update = now;
                window.set_title(&format!("Runa Sandbox - {:.1} FPS", fps));
            }
        }
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("Runa Sandbox");
            // use Arc.
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            let renderer = Renderer::new(window.clone());
            self.renderer = Some(renderer);
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        // Clear the "just" input states at the beginning of each event loop cycle
        // This ensures that "just pressed" events are only valid for one frame

        const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

        let current_time = Instant::now();
        let frame_time = (current_time - self.last_time).as_secs_f32().min(0.1);
        self.last_time = current_time;

        self.accumulator += frame_time;

        // Fixed timestep обновление
        while self.accumulator >= FIXED_TIMESTEP {
            self.input_state.camera = Some(self.camera.clone());

            // Обработка ввода во всех скриптах
            self.world.input(&self.input_state);

            self.world.update(FIXED_TIMESTEP);
            self.input_state.update_frame();

            self.accumulator -= FIXED_TIMESTEP;
        }

        let interpolation_factor = (self.accumulator / FIXED_TIMESTEP).min(1.0);

        // Запрашиваем перерисовку
        if let Some(window) = &self.window {
            window.request_redraw();
            self.render(interpolation_factor);
        }

        // Update console
        self.console.handle_input(&self.input_state);
        self.console.render(&mut self.queue, &self.camera);

        // Only process world input if console is not visible
        if !self.console.is_visible() {
            // Process input for all scripts
            self.world.input(&self.input_state);

            // Process interaction system
            self.interaction_system
                .update(&mut self.world, &self.input_state);
        } else {
            // When console is visible, still process input for the world
            // but scripts can check if console is visible and decide whether to respond
            self.world.input(&self.input_state);
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
                if let (Some(wgpu_ctx), Some(window)) =
                    (self.renderer.as_mut(), self.window.as_ref())
                {
                    wgpu_ctx.resize((new_size.width, new_size.height));
                    self.camera.viewport_size = (new_size.width, new_size.height);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_ctx) = self.renderer.as_mut() {
                    if let Ok(frame) = wgpu_ctx.surface.get_current_texture() {
                        frame.present();
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
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        self.input_state.keys_pressed.insert(key_code);
                        self.input_state.keys_just_pressed.insert(key_code);
                    } else {
                        self.input_state.keys_pressed.remove(&key_code);
                        self.input_state.keys_just_pressed.remove(&key_code);
                    }
                    if key_code == KeyCode::Backquote {
                        self.console.toggle();
                    }
                }

                // Handle text input for the console
                if event.state == ElementState::Pressed && self.console.is_visible() {
                    match event.logical_key {
                        winit::keyboard::Key::Character(c) => {
                            self.console.input_buffer.push_str(&c);
                        }
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
                            self.console.input_buffer.push_str("    "); // Insert 4 spaces for tab
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input_state.mouse_position = (position.x as f32, position.y as f32);
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    self.input_state.mouse_wheel_delta = y;
                }
                _ => {}
            },

            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed {
                    self.input_state.mouse_buttons_pressed.insert(button);
                    self.input_state.mouse_buttons_just_pressed.insert(button);
                } else {
                    self.input_state.mouse_buttons_pressed.remove(&button);
                    self.input_state.mouse_buttons_just_pressed.remove(&button);
                }
            }

            _ => (),
        }
    }
}

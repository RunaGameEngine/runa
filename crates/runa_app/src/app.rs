use std::sync::Arc;
use std::time::Instant;

use runa_core::components::Camera2D;
use runa_core::input::InputState;
use runa_core::ocs::World;
use runa_core::systems::InteractionSystem;
use runa_core::Console;
use runa_render::Renderer;
use runa_render_api::RenderQueue;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

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
    pub camera: Camera2D,
    pub world: World,

    pub last_time: Instant,
    pub accumulator: f32,
    pub frame_count: u32,
    pub current_fps: f32,
    pub last_fps_update: Instant,
    pub interaction_system: InteractionSystem,

    pub console: Console,

    pub config: RunaWindowConfig,
}

impl<'window> App<'window> {
    fn toggle_fullscreen(&mut self) {
        if let Some(window) = &self.window {
            self.config.fullscreen = !self.config.fullscreen;

            if self.config.fullscreen {
                // Open fullscreen
                let fullscreen = Some(Fullscreen::Borderless(window.current_monitor()));
                window.set_fullscreen(fullscreen);
            } else {
                // Close fullscreen
                window.set_fullscreen(None);
            }
        }
    }

    fn render(&mut self) {
        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            // Clear queue
            self.queue.clear();

            let interpolation_factor = (self.accumulator / (1.0 / 60.0)).min(1.0);

            // Compile render commands
            self.world.render(&mut self.queue, interpolation_factor);

            // Rendering
            renderer.draw(&self.queue, self.camera.matrix(), self.camera.virtual_size);

            // Update FPS
            self.frame_count += 1;
            let now = Instant::now();
            if now.duration_since(self.last_fps_update).as_secs_f32() >= 1.0 {
                self.current_fps = self.frame_count as f32
                    / now.duration_since(self.last_fps_update).as_secs_f32();
                self.frame_count = 0;
                self.last_fps_update = now;
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
            let win_attr =
                Window::default_attributes().with_title(&format!("{}", self.config.title));
            // use Arc.
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );

            if let Some(icon_path) = &self.config.window_icon {
                match runa_asset::load_window_icon(icon_path) {
                    Ok(icon) => {
                        window.set_window_icon(Some(icon));
                        println!("✅ Window icon loaded: {}", icon_path);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to load window icon '{}': {}", icon_path, e);
                    }
                }
            } else {
                match runa_asset::load_window_icon(
                    "D:/coding/projects/runa-engine/crates/runa_app/assets/icon.png",
                ) {
                    Ok(icon) => {
                        window.set_window_icon(Some(icon));
                    }
                    Err(e) => {
                        let yellow = "\x1b[33m";
                        let clear = "\x1B[0m";

                        eprintln!(
                            "{}runa_warning{}: Failed to load window icon: {}",
                            yellow, clear, e
                        );
                    }
                }
            }

            if self.config.fullscreen {
                // Open fullscreen
                let fullscreen = Some(Fullscreen::Borderless(window.current_monitor()));
                window.set_fullscreen(fullscreen);
            } else {
                // Close fullscreen
                window.set_fullscreen(None);
            }
            self.window = Some(window.clone());
            let renderer = Renderer::new(window.clone(), self.config.vsync);
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
            {
                let mut input_state = InputState::current_mut();
                input_state.camera = Some(self.camera.clone());
            } // Release the lock immediately after setting the camera

            self.world.update(FIXED_TIMESTEP);
            InputState::update_frame();

            self.accumulator -= FIXED_TIMESTEP;
        }

        // Update console
        self.console.handle_input();
        self.console.render(&mut self.queue, &self.camera);

        // Only process world input if console is not visible
        if !self.console.is_visible() {
            // Process interaction system
            self.interaction_system.update(&mut self.world);
        } else {
            // When console is visible, still process input for the world
            // but scripts can check if console is visible and decide whether to respond
        }

        // Запрашиваем перерисовку
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
                if let (Some(wgpu_ctx), Some(window)) =
                    (self.renderer.as_mut(), self.window.as_ref())
                {
                    wgpu_ctx.resize((new_size.width, new_size.height));
                    self.camera.viewport_size = (new_size.width, new_size.height);
                    self.camera.update_aspect_correction();
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
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
                    let mut input_state = InputState::current_mut();
                    if event.state == ElementState::Pressed {
                        input_state.keys_pressed.insert(key_code);
                        input_state.keys_just_pressed.insert(key_code);
                    } else {
                        input_state.keys_pressed.remove(&key_code);
                        input_state.keys_just_pressed.remove(&key_code);
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
                    input_state.mouse_buttons_just_pressed.remove(&button);
                }
            }
            _ => (),
        }
    }
}

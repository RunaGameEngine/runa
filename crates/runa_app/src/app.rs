use std::sync::Arc;
use std::time::Instant;

use runa_core::components::Camera;
use runa_core::input::InputState;
use runa_core::ocs::World;
use runa_core::systems::InteractionSystem;
use runa_core::{glam, Console};
use runa_render::Renderer;
use runa_render_api::RenderQueue;

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
    pub camera: Camera,
    pub camera_matrix_override: Option<glam::Mat4>, // For Camera3D
    pub active_camera_set: bool,                    // True if ActiveCamera was manually set
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
        runa_core::input_system::toggle_fullscreen();
        self.config.fullscreen = runa_core::input_system::is_fullscreen().unwrap_or(false);
    }

    fn render(&mut self) {
        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            // Clear queue
            self.queue.clear();

            let interpolation_factor = (self.accumulator / (1.0 / 60.0)).min(1.0);

            // Compile render commands from world
            self.world.render(&mut self.queue, interpolation_factor);

            // Render console on top (after clearing queue and world render)
            self.console.render(&mut self.queue, &self.camera);

            // Rendering - use 3D matrix if available, otherwise use 2D camera matrix
            let camera_matrix = self
                .camera_matrix_override
                .unwrap_or_else(|| self.camera.matrix());

            // Get ortho_size from active camera if available, otherwise use default camera
            let ortho_size = if self.active_camera_set {
                // Try to get ortho_size from the active camera object
                self.world
                    .objects
                    .iter()
                    .find_map(|obj| obj.get_component::<Camera>())
                    .map(|cam| cam.ortho_size)
                    .unwrap_or(self.camera.ortho_size)
            } else {
                self.camera.ortho_size
            };

            renderer.draw(&self.queue, camera_matrix, ortho_size);

            // Update FPS
            self.frame_count += 1;
            let now = Instant::now();
            if now.duration_since(self.last_fps_update).as_secs_f32() >= 1.0 {
                self.current_fps = self.frame_count as f32
                    / now.duration_since(self.last_fps_update).as_secs_f32();
                self.frame_count = 0;
                self.last_fps_update = now;
                self.config.title = runa_core::input_system::window_title()
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

    /// Sync camera - finds ActiveCamera or first available camera
    fn sync_camera(&mut self) {
        use runa_core::components::{ActiveCamera, Camera};

        // Reset state
        self.camera_matrix_override = None;
        self.active_camera_set = false;

        // First pass: Look for ActiveCamera marker
        for object in &mut self.world.objects {
            if object.get_component::<ActiveCamera>().is_some() {
                // This object is marked as active camera
                if let Some(camera) = object.get_component::<Camera>() {
                    self.camera_matrix_override = Some(camera.matrix());
                    self.active_camera_set = true;

                    // Update camera viewport size to match window
                    if let Some(renderer) = &self.renderer {
                        let w = renderer.surface_config.width.max(1);
                        let h = renderer.surface_config.height.max(1);
                        if let Some(cam) = object.get_component_mut::<Camera>() {
                            cam.viewport_size = (w, h);
                        }
                    }
                    return;
                }
                // ActiveCamera exists but no Camera
                break;
            }
        }

        // Second pass: Use first available Camera
        for object in &mut self.world.objects {
            if let Some(camera) = object.get_component::<Camera>() {
                self.camera_matrix_override = Some(camera.matrix());

                // Update camera viewport size to match window
                if let Some(renderer) = &self.renderer {
                    let w = renderer.surface_config.width.max(1);
                    let h = renderer.surface_config.height.max(1);
                    if let Some(cam) = object.get_component_mut::<Camera>() {
                        cam.viewport_size = (w, h);
                    }
                }
                return;
            }
        }

        // No camera found - log warning, render black screen
        if self.renderer.is_some() {
            eprintln!("[WARNING] No camera found in the scene");
        }
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_title(&format!("{}", self.config.title))
                .with_visible(false);
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
                match runa_asset::load_window_icon(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/icon.png"
                )) {
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

            runa_core::input_system::initialize_window_state(
                self.config.title.clone(),
                self.config.fullscreen,
                (self.config.width, self.config.height),
            );
            self.window = Some(window.clone());

            // Set window handle for input system (cursor control)
            runa_core::input_system::set_window_handle(&window);
            runa_core::input_system::set_window_size(self.config.width, self.config.height);
            runa_core::input_system::set_fullscreen(self.config.fullscreen);

            let renderer = Renderer::new(window.clone(), self.config.vsync);
            self.renderer = Some(renderer);
            self.sync_camera();
            window.request_redraw();
            window.set_visible(true);
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

        // Fixed timestep update
        while self.accumulator >= FIXED_TIMESTEP {
            {
                let mut input_state = InputState::current_mut();
                // Use active camera from world if available, otherwise use default camera
                let camera_to_use = if self.active_camera_set {
                    self.world
                        .objects
                        .iter()
                        .find_map(|obj| obj.get_component::<Camera>())
                        .cloned()
                        .unwrap_or(self.camera)
                } else {
                    self.camera
                };
                input_state.camera = Some(camera_to_use);
            } // Release the lock immediately after setting the camera

            // Update interaction system BEFORE world update so scripts can use hover state
            if !self.console.is_visible() {
                self.interaction_system.update(&mut self.world);
            }

            self.world.update(FIXED_TIMESTEP);
            InputState::update_frame();

            // Sync camera
            self.sync_camera();

            self.accumulator -= FIXED_TIMESTEP;
        }

        // Request a redraw
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
                    self.camera.resize(new_size.width, new_size.height);
                    self.config.width = new_size.width;
                    self.config.height = new_size.height;
                    runa_core::input_system::initialize_window_state(
                        runa_core::input_system::window_title()
                            .unwrap_or_else(|| self.config.title.clone()),
                        runa_core::input_system::is_fullscreen()
                            .unwrap_or(self.config.fullscreen),
                        (new_size.width, new_size.height),
                    );
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
                // First, let the console handle the input
                self.console.handle_keyboard(&event, event.state);

                // Update global input state (only if console is not visible)
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
                    input_state.mouse_buttons_just_pressed.remove(&button);
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
        // Handle relative mouse movement (for locked cursor)
        if let DeviceEvent::MouseMotion { delta } = event {
            let mut input_state = InputState::current_mut();
            input_state.mouse_delta = (delta.0 as f32, delta.1 as f32);
        }
    }
}

use std::time::Instant;

use runa_core::{components::Camera2D, systems::InteractionSystem, Console, World};
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

use crate::app::{App, RunaWindowConfig};

/// Default Runa App to start Application
pub struct RunaApp {}

pub trait GameState {}

impl RunaApp {
    /// Run Runa application with config (fullscreen ?, vsync ?, screensize ? etc.)
    pub fn run_with_config(
        mut world: World,
        config: RunaWindowConfig,
    ) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        // Initialize input
        runa_core::input::InputState::initialize();
        let interaction_system = InteractionSystem::new();

        let mut camera = Camera2D::new(320.0, 180.0);
        camera.resize(1280, 720);

        world.construct();
        world.start();

        let console = Console::new();

        let mut app = App {
            window: None,
            renderer: None,
            queue: runa_render_api::RenderQueue::new(),
            camera,
            world,
            last_time: Instant::now(),
            accumulator: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            interaction_system,
            console,
            config,
            current_fps: 0.0,
        };

        event_loop.run_app(&mut app)
    }

    /// Run Runa application with default config:
    /// title: "Runa Game".to_string(),
    /// width: 1280,
    /// height: 720,
    /// fullscreen: false,
    /// vsync: true
    pub fn run_default(mut world: World) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        runa_core::input::InputState::initialize();
        let interaction_system = InteractionSystem::new();

        let camera = Camera2D::new(320.0, 180.0);

        world.construct();
        world.start();

        let console = Console::new();

        let mut app = App {
            window: None,
            renderer: None,
            queue: runa_render_api::RenderQueue::new(),
            camera,
            world,
            last_time: Instant::now(),
            accumulator: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            interaction_system,
            console,
            config: RunaWindowConfig::default(),
            current_fps: 0.0,
        };

        event_loop.run_app(&mut app)
    }
}

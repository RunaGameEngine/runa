use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use runa_core::{components::Camera, systems::InteractionSystem, Console, World};
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
        world_rc: Rc<RefCell<World>>,
        config: RunaWindowConfig,
    ) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        // Initialize input
        runa_core::input::InputState::initialize();
        let interaction_system = InteractionSystem::new();

        // Default camera (will be overridden by world cameras if present)
        let camera = Camera::default();

        {
            let mut world = world_rc.borrow_mut();
            world.construct();
            world.start(world_rc.clone());
        }

        let console = Console::new();

        let mut app = App {
            window: None,
            renderer: None,
            queue: runa_render_api::RenderQueue::new(),
            camera,
            camera_matrix_override: None,
            active_camera_set: false,
            world_rc: world_rc.clone(),
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
    pub fn run_default(world_rc: Rc<RefCell<World>>) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        runa_core::input::InputState::initialize();
        let interaction_system = InteractionSystem::new();

        // Default camera (will be overridden by world cameras if present)
        let camera = Camera::default();

        {
            let mut world = world_rc.borrow_mut();
            world.construct();
            world.start(world_rc.clone());
        }

        let console = Console::new();

        let mut app = App {
            window: None,
            renderer: None,
            queue: runa_render_api::RenderQueue::new(),
            camera,
            camera_matrix_override: None,
            active_camera_set: false,
            world_rc: world_rc.clone(),
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

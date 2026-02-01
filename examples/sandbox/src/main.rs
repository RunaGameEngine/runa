use std::time::Instant;

use crate::app::App;

use runa_core::components::camera2d::Camera2D;
use runa_core::console::Console;
use runa_core::input::InputState;
use runa_core::systems::interaction_system::InteractionSystem;
use runa_render_api::queue::RenderQueue;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::player::Player;
use crate::tester1::RotatingSprite1;

mod app;
mod player;
mod tester1;

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let camera = Camera2D::new(320.0, 180.0); // виртуальный размер
    let input_state = InputState::default();
    let mut world = runa_core::ocs::world::World::default();
    let interaction_system = InteractionSystem::new();

    world.spawn(Box::new(RotatingSprite1::new()));
    world.spawn(Box::new(Player::new()));

    world.construct();
    world.start();

    let console = Console::new();

    let mut app = App {
        last_time: Instant::now(),
        accumulator: 0.0,
        frame_count: 0,
        last_fps_update: Instant::now(),
        window: None,
        renderer: None,
        queue: RenderQueue::new(),
        camera,
        world,
        is_fullscreen: false,
        input_state,
        interaction_system,
        console,
    };

    event_loop.run_app(&mut app)
}

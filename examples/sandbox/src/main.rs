use std::time::Instant;

use crate::app::App;

use runa_core::components::Camera2D;
use runa_core::systems::InteractionSystem;
use runa_core::Console;
use runa_render_api::RenderQueue;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::player::Player;
use crate::tester1::RotatingSprite1;
use crate::tilemap_tester::TilemapTester;

mod app;
mod player;
mod tester1;
mod tilemap_tester;

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    runa_core::input::InputState::initialize();
    let interaction_system = InteractionSystem::new();

    let camera = Camera2D::new(1280.0, 720.0);
    let mut world = runa_core::ocs::World::default();

    world.spawn(Box::new(TilemapTester::new()));
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
        interaction_system,
        console,
    };

    event_loop.run_app(&mut app)
}

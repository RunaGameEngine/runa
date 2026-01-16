use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

pub mod player;

use runa_render::renderer::{GpuContext, Renderer};
use runa_render_api::queue::RenderQueue;

fn main() {
    pollster::block_on(run());
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("Runa Sandbox")
        .build(&event_loop)
        .unwrap();
    let size = window.inner_size();

    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(&window).unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find adapter");

    // Создаём renderer, НО не захватываем window в closure
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");
    let context = GpuContext { device, queue };

    let surface_format = surface.get_capabilities(&adapter).formats[0];

    let mut renderer = Renderer::new(context, surface_format);

    // ───────────────── Render Queue ───────────
    let render_queue = RenderQueue::new();

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    let window_id = window.id(); // сохраняем ID
    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            window_id: event_window_id,
            ref event,
        } if window_id == event_window_id => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::RedrawRequested => {
                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        surface.configure(&renderer.context.device, &config);
                        return;
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = renderer.context.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Main Encoder"),
                    },
                );

                renderer.draw(&mut encoder, &view, &render_queue);
            }
            WindowEvent::Resized(size) => {
                config.width = size.width.max(1);
                config.height = size.height.max(1);
                surface.configure(&renderer.context.device, &config);
            }
            _ => (),
        },
        _ => (),
    })?;

    Ok(())
}

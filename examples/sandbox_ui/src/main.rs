use runa_engine::runa_app::{RunaApp, RunaWindowConfig};
use runa_engine::runa_core::components::ui::{CanvasSpace, UiRenderer};
use runa_engine::runa_core::components::Camera;
use runa_engine::runa_ecs;
use runa_engine::system;

#[system]
fn ui_builder(world: &mut runa_ecs::World) {
    for (_, ui) in world.query_mut::<runa_ecs::W<UiRenderer>>() {
        ui.clear();
        ui.vbox(|ui| {
            ui.add_text("Runa Engine UI Demo")
                .with_font_size(28)
                .with_text_color(0.0, 0.8, 1.0, 1.0);

            ui.add_text("This is a sandbox UI example.")
                .with_font_size(16)
                .with_text_color(0.8, 0.8, 0.8, 1.0);

            ui.add_button(Some("Click Me"), Some(Box::new(|| {
                println!("Button clicked!");
            })))
            .with_background(0.2, 0.5, 0.8, 1.0)
            .with_size(160.0, 40.0);

            ui.add_slider()
                .with_slider_value(0.5)
                .with_slider_range(0.0, 1.0)
                .with_size(200.0, 24.0);
        })
        .with_background(0.1, 0.1, 0.15, 0.9)
        .with_padding(16.0, 16.0, 16.0, 16.0)
        .with_gap(8.0)
        .with_pos(40.0, 40.0)
        .with_size(300.0, 400.0);
    }
}

fn main() {
    let mut world = runa_ecs::World::new();

    world.spawn((Camera::new_orthographic(320.0, 180.0),));

    world.spawn((UiRenderer::new(CanvasSpace::Screen),));

    let config = RunaWindowConfig {
        title: "Runa UI Demo".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_world(config, world);
}

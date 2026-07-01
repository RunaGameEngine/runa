use runa_core::components::ui::CanvasSpace;
use runa_engine::prelude::*;

fn main() {
    let engine = Engine::new();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();
        let mut ui = UiRenderer::new(CanvasSpace::Camera);

        ui.vbox(|ui| {
            ui.add_text("egui-style UI")
                .with_font_size(28)
                .with_text_color(1.0, 1.0, 1.0, 1.0);

            ui.add_text("Nested containers with closures")
                .with_font_size(16)
                .with_text_color(0.8, 0.8, 1.0, 1.0);

            ui.hbox(|ui| {
                ui.add_button(Some("Click"), None)
                    .with_on_click(|| println!("Button clicked!"))
                    .with_size(54.0, 36.0)
                    .with_background(0.3, 0.5, 0.7, 1.0);
            });

            ui.add_slider()
                .with_slider_range(0.0, 100.0)
                .with_slider_value(50.0)
                .with_size(300.0, 30.0)
                .with_background(0.15, 0.15, 0.2, 0.8);
        });

        world.spawn_bundle((
            Camera::default(),
            ActiveCamera,
            ui,
        ));
    }

    let cfg = RunaWindowConfig {
        title: "".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: false,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(world_rc, cfg);
}

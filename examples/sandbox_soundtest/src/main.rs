use runa_engine::runa_app::{RunaApp, RunaWindowConfig};

fn main() {
    let config = RunaWindowConfig {
        title: "Runa Sound Test".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_config(config);
}

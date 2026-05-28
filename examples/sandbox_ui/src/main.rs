use runa_core::components::ui::ImageProps;
use runa_core::{
    components::ui::{CanvasSpace, TextAlign, TextProps},
    components::{ActiveCamera, Camera, UiRenderer},
    glam,
    ocs::Object,
};
use runa_engine::{runa_app::RunaApp, Engine};

fn main() {
    let engine = Engine::new();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();

        let mut ui_renderer = UiRenderer::new(CanvasSpace::Screen);
        let message = "Hello UI yupiiiiii".to_string();

        // Pick a reasonable default font size (engine will provide viewport via active camera)
        let font_size: u16 = 48; // sensible default for demo

        let text_node = ui_renderer.add_text(
            ui_renderer.root(),
            TextProps {
                text: message,
                font: None,
                font_size,
                color: [1.0, 1.0, 1.0, 1.0],
                line_height: None,
                align: TextAlign::Center,
            },
        );

        // Load image handle (returns Handle<TextureAsset>)
        let image_handle = runa_asset::load_image!("assets/Charactert.png");
        // Use full texture UVs by default for visibility testing
        let uv = [0.0_f32, 0.0_f32, 1.0_f32, 1.0_f32];

        // Place image — do not manually set computed rect. Use layout to position relative to viewport.
        let image_id = ui_renderer.add_image(
            ui_renderer.root(),
            ImageProps {
                texture: Some(image_handle),
                tint: [1.0, 1.0, 1.0, 1.0],
                uv,
            },
        );

        // Configure layout: center top for text and below it for image
        if let Some(node) = ui_renderer.node_mut(text_node) {
            node.layout.anchor = runa_core::components::ui::Anchor::TopCenter;
            node.layout.position = glam::Vec2::new(0.0, 20.0); // 20px from top
                                                               // allow layout to compute size based on font_size
        }

        if let Some(node) = ui_renderer.node_mut(image_id) {
            node.layout.anchor = runa_core::components::ui::Anchor::TopCenter;
            node.layout.position = glam::Vec2::new(0.0, 80.0); // below text
                                                               // uv already provided; layout will size image based on texture
        }

        let mut camera_object = Object::new("MainCamera");
        camera_object.add_component(Camera::default());
        camera_object.add_component(ActiveCamera);
        camera_object.add_component(ui_renderer);
        world.spawn(camera_object);
    }

    let _ = RunaApp::run_default(world_rc);
}

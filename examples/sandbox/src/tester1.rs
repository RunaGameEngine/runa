use glam::Vec2;
use runa_core::components::cursor_interactable::CursorInteractable;
use runa_core::components::sprite_renderer::SpriteRenderer;
use runa_core::components::transform::Transform;
use runa_core::ocs::object::Object;
use runa_core::ocs::script::Script;
use winit::keyboard::KeyCode;

pub struct RotatingSprite1 {}

impl RotatingSprite1 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Script for RotatingSprite1 {
    fn construct(&self, _object: &mut runa_core::ocs::object::Object) {
        let mut interactable = CursorInteractable::new(2.0, 2.0);
        interactable.set_on_hover_enter(|| {
            println!("🖱️ HOVER ENTER");
        });
        interactable.set_on_hover_exit(|| {
            println!("🖱️ HOVER EXIT");
        });

        _object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::loader::load_image("assets/Tester1.png")),
            })
            .add_component(interactable); // интерактивная коллизия для курсора
    }

    fn start(&mut self, _object: &mut Object) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            transform.position = Vec2 { x: 1.0, y: 1.0 };
            transform.scale = Vec2 { x: 1.0, y: 1.0 };
        }
    }

    fn update(&mut self, _object: &mut Object, _dt: f32) {}

    fn input(&mut self, _object: &mut Object, _input: &runa_core::input::InputState) {}
}

use runa_core::components::CursorInteractable;
use runa_core::components::SpriteRenderer;
use runa_core::components::Transform;
use runa_core::glam::Vec3;
use runa_core::input_system::*;
use runa_core::ocs::Object;
use runa_core::ocs::Script;

pub struct RotatingSprite1 {}

impl RotatingSprite1 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Script for RotatingSprite1 {
    fn construct(&self, _object: &mut Object) {
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
                texture: Some(runa_asset::load_image!("assets/art/Tester1.png")),
            })
            .add_component(interactable); // интерактивная коллизия для курсора
    }

    fn start(&mut self, _object: &mut Object) {
        if let Some(transform) = _object.get_component_mut::<Transform>() {
            transform.position = Vec3 {
                x: 0.0,
                y: 4.0,
                z: 0.0,
            };
        }
    }

    fn update(&mut self, _object: &mut Object, _dt: f32, _world: &mut runa_core::World) {
        if let Some(ci) = &_object.get_component::<CursorInteractable>() {
            if ci.is_hovered && Input::is_mouse_button_pressed(MouseButton::Left) {
                if let Some(transform) = _object.get_component_mut::<Transform>() {
                    transform.position = Input::get_mouse_world_position().unwrap_or_default();
                }
            }
        }
    }
}

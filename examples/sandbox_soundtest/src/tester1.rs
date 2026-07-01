use runa_core::components::*;
use runa_core::glam::Vec3;
use runa_core::input::InputState;
use runa_core::ocs::{Object, Script, ScriptContext};
use winit::event::MouseButton;

pub struct RotatingSprite1;

impl RotatingSprite1 {
    pub fn new() -> Self {
        Self
    }
}

impl Script for RotatingSprite1 {
    fn start(&mut self, ctx: &mut ScriptContext) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position = Vec3::new(0.0, 4.0, 0.0);
        }
    }

    fn update(&mut self, ctx: &mut ScriptContext, _dt: f32) {
        if let Some(ci) = ctx.get_component::<CursorInteractable>() {
            if ci.is_hovered && InputState::is_mouse_button_pressed(MouseButton::Left) {
                if let Some(transform) = ctx.get_component_mut::<Transform>() {
                    transform.position = InputState::get_mouse_world_position().unwrap_or_default();
                }
            }
        }
    }
}

pub fn create_rotating_sprite() -> Object {
    let mut interactable = CursorInteractable::new(2.0, 2.0);
    interactable.set_on_hover_enter(|| {
        println!("HOVER ENTER");
    });
    interactable.set_on_hover_exit(|| {
        println!("HOVER EXIT");
    });

    Object::new("Rotating Sprite")
        .with(SpriteRenderer::from_path("assets/art/Tester1.png"))
        .with(interactable)
        .with(RotatingSprite1::new())
}

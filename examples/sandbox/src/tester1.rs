use runa_core::components::{CursorInteractable, SpriteRenderer, Transform};
use runa_core::glam::Vec3;
use runa_core::input_system::*;
use runa_core::ocs::{Object, Script, ScriptContext, World};
use runa_engine::{RunaArchetype, RunaScript};

#[derive(Default, RunaScript)]
pub struct RotatingSprite1;

impl RotatingSprite1 {
    pub fn new() -> Self {
        Self
    }
}

impl Script for RotatingSprite1 {
    fn start(&mut self, ctx: &mut ScriptContext) {
        ctx.subscribe_to_event::<crate::player::EventChangedDirectionX>(|_| {
            println!("Direction Changed");
        });

        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            transform.position = Vec3::new(0.0, 4.0, 0.0);
        }
    }

    fn update(&mut self, ctx: &mut ScriptContext, _dt: f32) {
        if let Some(ci) = ctx.get_component::<CursorInteractable>() {
            if ci.is_hovered && Input::is_mouse_button_pressed(MouseButton::Left) {
                if let Some(transform) = ctx.get_component_mut::<Transform>() {
                    transform.position = Input::get_mouse_world_position().unwrap_or_default();
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
        .with(SpriteRenderer {
            texture: Some(runa_asset::load_image!("assets/art/Tester1.png")),
            texture_path: Some("assets/art/Tester1.png".to_string()),
            pixels_per_unit: 16.0,
            uv_rect: SpriteRenderer::FULL_UV_RECT,
        })
        .with(interactable)
        .with(RotatingSprite1::new())
}

#[derive(RunaArchetype)]
#[runa(name = "rotating_sprite")]
pub struct RotatingSpriteArchetype;

impl RotatingSpriteArchetype {
    pub fn create(world: &mut World) -> u64 {
        world.spawn(create_rotating_sprite())
    }
}

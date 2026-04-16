use runa_core::{
    components::{Collider2D, SpriteRenderer, Transform},
    glam::Vec3,
    ocs::{Object, Script},
};

pub struct ColliderDemoBox;

impl ColliderDemoBox {
    pub fn new() -> Self {
        Self
    }
}

impl Script for ColliderDemoBox {
    fn construct(&self, object: &mut Object) {
        object
            .add_component(Transform::default())
            .add_component(SpriteRenderer {
                texture: Some(runa_asset::load_image!("assets/art/Tester2.png")),
                texture_path: Some("assets/art/Tester2.png".to_string()),
            })
            .add_component(Collider2D::new(16.0, 16.0));
    }

    fn start(&mut self, object: &mut Object) {
        if let Some(transform) = object.get_component_mut::<Transform>() {
            transform.position = Vec3::new(32.0, 0.0, 0.0);
        }
    }
}

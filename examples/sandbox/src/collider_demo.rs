use runa_core::{
    components::{Collider2D, SpriteRenderer, Transform},
    glam::Vec3,
    ocs::Object,
};

pub fn create_collider_demo_box() -> Object {
    let mut transform = Transform::default();
    transform.position = Vec3::new(32.0, 0.0, 0.0);

    Object::new("Collider Demo Box")
        .with(transform)
        .with(SpriteRenderer::from_path("assets/art/Tester2.png"))
        .with(Collider2D::new(16.0, 16.0))
}

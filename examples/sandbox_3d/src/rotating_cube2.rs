use runa_core::components::{Mesh, MeshRenderer, Transform};
use runa_core::glam::{Quat, Vec3};
use runa_core::ocs::{Object, Script, ScriptContext};

pub struct RotatingCube2 {
    rotation_speed: f32,
}

impl RotatingCube2 {
    pub fn new() -> Self {
        Self {
            rotation_speed: 0.5,
        }
    }
}

impl Default for RotatingCube2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Script for RotatingCube2 {
    fn update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        if let Some(transform) = ctx.get_component_mut::<Transform>() {
            let rotation = Quat::from_rotation_y(self.rotation_speed * dt);
            transform.rotation *= rotation;
        }
    }
}

pub fn create_rotating_cube2() -> Object {
    Object::new("Rotating Cube 2")
        .with(Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(-2.0, 2.0, 2.0),
            previous_position: Vec3::ZERO,
            previous_rotation: Quat::IDENTITY,
        })
        .with(MeshRenderer::new(Mesh::cube(1.0)))
        .with(RotatingCube2::new())
}

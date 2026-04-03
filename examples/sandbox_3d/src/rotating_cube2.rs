use runa_core::components::{Mesh, MeshRenderer, Transform};
use runa_core::glam::{Quat, Vec3};
use runa_core::ocs::{Object, Script};

/// Simple 3D cube to verify mesh rendering works
pub struct RotatingCube2 {
    rotation_speed: f32,
}

impl RotatingCube2 {
    pub fn new() -> Self {
        Self {
            rotation_speed: 0.5, // radians per second
        }
    }
}

impl Script for RotatingCube2 {
    fn construct(&self, object: &mut Object) {
        // Add transform - position 3 units in front of camera
        object.add_component(Transform {
            position: Vec3::new(0., 0., 0.), // 2 units in front of camera at (0,0,5)
            rotation: Quat::IDENTITY,
            scale: Vec3::new(-2.0, 2.0, 2.0),
            previous_position: Vec3::ZERO,
            previous_rotation: Quat::IDENTITY,
        });

        // Add 3D mesh (cube)
        let mesh = Mesh::cube(1.);
        object.add_component(MeshRenderer::new(mesh));
    }

    fn update(&mut self, object: &mut Object, dt: f32) {
        if let Some(transform) = object.get_component_mut::<Transform>() {
            // Rotate around Y axis
            let rotation = Quat::from_rotation_y(self.rotation_speed * dt);
            transform.rotation = transform.rotation * rotation;
        }
    }
}

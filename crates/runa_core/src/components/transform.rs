use glam::{Mat4, Vec2};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position.extend(0.0))
            * Mat4::from_rotation_z(self.rotation)
            * Mat4::from_scale(self.scale.extend(1.0))
    }
}

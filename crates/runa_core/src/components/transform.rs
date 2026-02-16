use glam::{Quat, Vec3};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    // for interpolation for fixedframe
    pub previous_position: Vec3,
    pub previous_rotation: Quat,
}

impl Transform {
    /// position: Vec2::ZERO, rotation: Quat::IDENTITY, scale: Vec2::ONE,
    pub fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            previous_position: Vec3::ZERO,
            previous_rotation: Quat::IDENTITY,
        }
    }

    pub fn rotate_x(&mut self, angle: f32) {
        self.rotation *= Quat::from_rotation_x(angle.to_radians());
    }

    pub fn rotate_y(&mut self, angle: f32) {
        self.rotation *= Quat::from_rotation_y(angle.to_radians());
    }

    pub fn rotate_z(&mut self, angle: f32) {
        self.rotation *= Quat::from_rotation_z(angle.to_radians());
    }

    pub fn prepare_for_update(&mut self) {
        self.previous_position = self.position;
        self.previous_rotation = self.rotation;
    }
}

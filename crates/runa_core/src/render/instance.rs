use glam::Mat4;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model: [[f32; 4]; 4],
}

impl InstanceData {
    pub fn from_transform(transform: &crate::math::transform::Transform) -> Self {
        Self {
            model: transform.matrix().to_cols_array_2d(),
        }
    }
}

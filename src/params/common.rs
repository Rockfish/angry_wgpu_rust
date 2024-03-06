use glam::Vec4;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionLight {
    pub direction: Vec4,
    pub color: Vec4,
}
impl Default for DirectionLight {
    fn default() -> Self {
        DirectionLight {
            direction: Default::default(),
            color: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    pub world_pos: Vec4,
    pub color: Vec4,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            world_pos: Default::default(),
            color: Default::default(),
        }
    }
}

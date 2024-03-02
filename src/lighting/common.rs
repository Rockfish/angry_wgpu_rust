use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionLight {
    pub direction: Vec3,
    pub color: Vec3,
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
    pub world_pos: Vec3,
    pub color: Vec3,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            world_pos: Default::default(),
            color: Default::default(),
        }
    }
}

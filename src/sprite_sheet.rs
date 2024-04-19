use glam::Vec3;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::Material;
use wgpu::{BindGroup, Buffer};

use crate::render::buffers::{create_buffer_bind_group, create_uniform_bind_group_layout, create_uniform_buffer_init, get_or_create_bind_group_layout};

pub const SPRITE_BIND_GROUP_LAYOUT: &str = "sprite_bind_group_layout";

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteSheetUniform {
    pub num_columns: f32,
    pub time_per_sprite: f32,
}

#[derive(Debug)]
pub struct SpriteSheet {
    pub material: Material,
    pub uniform: SpriteSheetUniform,
    pub uniform_buffer: Buffer,
    pub uniform_bind_group: BindGroup,
}

impl SpriteSheet {
    pub fn new(context: &mut GpuContext, material: Material, num_columns: f32, time_per_sprite: f32) -> Self {
        let uniform = SpriteSheetUniform { num_columns, time_per_sprite };
        let buffer = create_uniform_buffer_init(context, &[uniform], "sprite sheet uniform");
        let layout = get_or_create_bind_group_layout(context, SPRITE_BIND_GROUP_LAYOUT, create_uniform_bind_group_layout);
        let bind_group = create_buffer_bind_group(context, &layout, &buffer, "sprite sheet uniform bind");

        Self {
            material,
            uniform,
            uniform_buffer: buffer,
            uniform_bind_group: bind_group,
        }
    }
}

pub struct SpriteSheetSprite {
    pub world_position: Vec3,
    pub age: f32,
}

impl SpriteSheetSprite {
    pub fn new(world_position: Vec3) -> Self {
        Self { world_position, age: 0.0 }
    }
}

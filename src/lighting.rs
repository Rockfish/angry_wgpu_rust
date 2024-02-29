use glam::{Mat4, Vec3};
use spark_gap::gpu_context::GpuContext;
use wgpu::{BindGroup, BindGroupLayout, Buffer};
use wgpu::util::DeviceExt;

pub const GAME_LIGHTING_BIND_GROUP_LAYOUT: &str = "game_lighting_bind_group_layout";

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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GameLightingUniform {
    pub direction_light: DirectionLight,
    pub point_light: PointLight,
    pub aim_rotation: Mat4,
    pub light_space_matrix: Mat4,
    pub view_position: Vec3,
    pub ambient_color: Vec3,
    pub depth_mode: i32,
    pub use_point_light: i32,
    pub use_light: i32,
    pub use_emissive: i32,
    pub(crate) _pad: [f32; 6],
}

impl GameLightingUniform {
    pub fn new() -> Self {
        Self {
            direction_light: Default::default(),
            point_light: Default::default(),
            aim_rotation: Default::default(),
            light_space_matrix: Default::default(),
            view_position: Default::default(),
            ambient_color: Default::default(),
            depth_mode: 0,
            use_point_light: 0,
            use_light: 0,
            use_emissive: 0,
            _pad: [0.0; 6],
        }
    }
}

pub struct GameLightingHandler {
    pub uniform: GameLightingUniform,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
}

impl GameLightingHandler {
    pub fn new(context: &mut GpuContext, lighting_uniform: GameLightingUniform) -> Self {

        // let lighting_uniform = game_lighting.get_camera_uniform();

        let uniform_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Lighting Buffer"),
            contents: bytemuck::cast_slice(&[lighting_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
        );

        if !context.bind_layout_cache.contains_key(GAME_LIGHTING_BIND_GROUP_LAYOUT) {
            let layout = create_lighting_bind_group_layout(context);
            context.bind_layout_cache.insert(String::from(GAME_LIGHTING_BIND_GROUP_LAYOUT), layout);
        }

        let bind_group_layout = context.bind_layout_cache.get(GAME_LIGHTING_BIND_GROUP_LAYOUT).unwrap();

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("game_lighting_bind_group"),
        });

        Self {
            uniform: lighting_uniform,
            uniform_buffer,
            bind_group
        }
    }

    pub fn update_lighting(&self, context: &GpuContext) {
        // let camera_uniform = camera_controller.get_camera_uniform();
        context
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

fn create_lighting_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
    context.device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(GAME_LIGHTING_BIND_GROUP_LAYOUT),
    })
}

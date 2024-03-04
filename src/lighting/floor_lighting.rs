use glam::{Mat4, Vec3};
use spark_gap::gpu_context::GpuContext;
use wgpu::{BindGroup, BindGroupLayout, Buffer};
use wgpu::util::DeviceExt;
use crate::lighting::common::{DirectionLight, PointLight};

pub const FLOOR_LIGHTING_BIND_GROUP_LAYOUT: &str = "floor_lighting_bind_group_layout";

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FloorLightingUniform {
    pub direction_light: DirectionLight,
    pub point_light: PointLight,
    pub light_space_matrix: Mat4,
    pub ambient_color: Vec3,
    pub view_position: Vec3,
    pub use_lighting: i32,
    pub use_specular: i32,
    pub use_point_light: i32,
    pub(crate) _pad: [f32; 7],
}

pub struct FloorLightingHandler {
    pub uniform: FloorLightingUniform,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
}

impl FloorLightingHandler {
    pub fn new(context: &mut GpuContext, lighting_uniform: FloorLightingUniform) -> Self {

        let uniform_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Lighting Buffer"),
                contents: bytemuck::cast_slice(&[lighting_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        if !context.bind_layout_cache.contains_key(FLOOR_LIGHTING_BIND_GROUP_LAYOUT) {
            let layout = create_floor_lighting_bind_group_layout(context);
            context.bind_layout_cache.insert(String::from(FLOOR_LIGHTING_BIND_GROUP_LAYOUT), layout.into());
        }

        let bind_group_layout = context.bind_layout_cache.get(FLOOR_LIGHTING_BIND_GROUP_LAYOUT).unwrap();

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("floor_lighting_bind_group"),
        });

        Self {
            uniform: lighting_uniform,
            uniform_buffer,
            bind_group
        }
    }

    pub fn update_lighting(&self, context: &GpuContext) {
        context
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

fn create_floor_lighting_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
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
            label: Some(FLOOR_LIGHTING_BIND_GROUP_LAYOUT),
        })
}


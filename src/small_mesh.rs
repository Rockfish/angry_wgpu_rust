use std::rc::Rc;
use glam::Mat4;
use spark_gap::gpu_context::GpuContext;
use wgpu::{BindGroup, BindGroupLayout, Buffer};
use wgpu::util::DeviceExt;

pub const SMALL_MESH_BIND_GROUP_LAYOUT: &str = "small_mesh_bind_group_layout";

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SmallMeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct SmallMesh {
    pub vertex_buffer: Rc<wgpu::Buffer>,
    pub num_elements: u32,
}

impl SmallMesh {
    pub fn vertex_description() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SmallMeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // vertices
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tex coords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub fn get_or_create_bind_group_layout<'a>(context: &'a mut GpuContext, layout_name: &'a str, create_func: fn(&GpuContext) -> BindGroupLayout) -> &'a BindGroupLayout {
    if !context.bind_layout_cache.contains_key(layout_name) {
        let layout = create_func(context);
        context.bind_layout_cache.insert(String::from(layout_name), layout);
    }

    context.bind_layout_cache.get(layout_name).unwrap()
}

pub fn create_small_mesh_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            // 0: model transform
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("small mesh bind group layout"),
    })
}

pub fn create_transform_buffer(context: &mut GpuContext, label: &str, data: &Mat4) -> Buffer {
    context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&data.to_cols_array()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

// small buffer bind group?
pub fn create_small_mesh_bind_group(
    context: &GpuContext,
    bind_group_layout: &BindGroupLayout,
    transform_buffer: &Buffer,
) -> BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            },
        ],
        label: Some("small mesh bind group"),
    })
}


use std::rc::Rc;
use glam::Mat4;
use spark_gap::gpu_context::GpuContext;
use wgpu::{BindGroup, BindGroupLayout, Buffer};
use wgpu::util::DeviceExt;

pub const TRANSFORM_BIND_GROUP_LAYOUT: &str = "transform bind group layout";

pub fn create_uniform_buffer<T: bytemuck::Pod>(context: &GpuContext, uniform: &[T], label: &str) -> Buffer {
    context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_uniform_box_buffer<T: bytemuck::Pod>(context: &GpuContext, uniform: &Box<[T]>, label: &str) -> Buffer {
    context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_vertex_buffer<T: bytemuck::Pod>(context: &GpuContext, uniform: &[T], label: &str) -> Buffer {
    context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(uniform),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

pub fn create_vertex_box_buffer<T: bytemuck::Pod>(context: &GpuContext, uniform: &Box<[T]>, label: &str) -> Buffer {
    context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(uniform),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

pub fn create_mat4_buffer(context: &mut GpuContext, data: &Mat4, label: &str) -> Buffer {
    context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::cast_slice(&data.to_cols_array()),
        label: Some(label),
    })
}

pub fn update_uniform_buffer<T: bytemuck::Pod>(context: &GpuContext, buffer: &Buffer, uniform: &[T; 1]) {
    context
        .queue
        .write_buffer(buffer, 0, bytemuck::cast_slice(uniform));
}

pub fn update_uniform_box_buffer<T: bytemuck::Pod>(context: &GpuContext, buffer: &Buffer, uniform: &Box<[T]>) {
    context
        .queue
        .write_buffer(buffer, 0, bytemuck::cast_slice(uniform));
}

pub fn update_mat4_buffer(context: &GpuContext, buffer: &Buffer, data: &Mat4) {
    context
        .queue
        .write_buffer(buffer, 0, bytemuck::cast_slice(&data.to_cols_array()));
}

pub fn get_or_create_bind_group_layout(context: &mut GpuContext, layout_name: &str, create_func: fn(&GpuContext, &str) -> BindGroupLayout) -> Rc<BindGroupLayout> {
    if !context.bind_layout_cache.contains_key(layout_name) {
        let layout = create_func(context, layout_name);
        context.bind_layout_cache.insert(String::from(layout_name), layout.into());
    }

    context.bind_layout_cache.get(layout_name).unwrap().clone()
}

pub fn create_vertex_bind_group_layout(context: &GpuContext, label: &str) -> BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some(label),
    })
}

pub fn create_uniform_bind_group_layout(context: &GpuContext, label: &str) -> BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some(label),
    })
}

pub fn create_buffer_bind_group(
    context: &GpuContext,
    bind_group_layout: &BindGroupLayout,
    buffer: &Buffer,
    label: &str,
) -> BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },
        ],
        label: Some(label),
    })
}


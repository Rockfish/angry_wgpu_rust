use glam::Mat4;
use spark_gap::gpu_context::GpuContext;
use std::rc::Rc;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SmallMeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct SmallMesh {
    pub vertex_buffer: Rc<Buffer>,
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
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                },
                // tex coords
                wgpu::VertexAttribute {
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                },
            ],
        }
    }
}

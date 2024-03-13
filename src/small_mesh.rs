use std::rc::Rc;
use glam::Mat4;
use spark_gap::gpu_context::GpuContext;
use wgpu::{BindGroup, BindGroupLayout, Buffer};
use wgpu::util::DeviceExt;


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




use std::mem;

use glam::{Vec3, Vec4};
use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::MATERIAL_BIND_GROUP_LAYOUT;
use wgpu::{IndexFormat, RenderPass, RenderPipeline};

use crate::bullets::BulletSystem;
use crate::load_shader;
use crate::small_mesh::SmallMesh;
use crate::world::World;

pub fn create_bullet_shader_pipeline(context: &GpuContext) -> RenderPipeline {
    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("bullet shader pipeline layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,
            material_bind_group_layout, // diffuse
        ],
        push_constant_ranges: &[],
    });

    let shader = context.device.create_shader_module(load_shader!("bullet_shader.wgsl").into());

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let instance_vec3_description = wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vec3>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x3,
        }],
    };

    let instance_vec4_description = wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vec4>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32x4,
        }],
    };

    let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bullet Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[SmallMesh::vertex_description(), instance_vec3_description, instance_vec4_description],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: swapchain_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    render_pipeline
}

pub fn render_bullets<'a>(world: &'a World, mut render_pass: RenderPass<'a>, bullet_system: &'a BulletSystem) -> RenderPass<'a> {
    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);
    render_pass.set_bind_group(1, &bullet_system.bullet_material.bind_group, &[]);

    render_pass.set_vertex_buffer(0, bullet_system.vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, bullet_system.bullet_positions_buffer.slice(..));
    render_pass.set_vertex_buffer(2, bullet_system.bullet_rotations_buffer.slice(..));

    render_pass.set_index_buffer(bullet_system.index_buffer.slice(..), IndexFormat::Uint32);

    render_pass.draw_indexed(0..12, 0, 0..bullet_system.bullet_positions.len() as u32);

    render_pass
}

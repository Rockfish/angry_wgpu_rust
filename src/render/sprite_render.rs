use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::MATERIAL_BIND_GROUP_LAYOUT;
use wgpu::{RenderPass, RenderPipeline};

use crate::load_shader;
use crate::muzzle_flash::MuzzleFlash;
use crate::render::buffers::TRANSFORM_BIND_GROUP_LAYOUT;
use crate::small_mesh::SmallMesh;
use crate::sprite_sheet::SPRITE_BIND_GROUP_LAYOUT;
use crate::world::World;

pub fn create_sprite_shader_pipeline(context: &GpuContext) -> RenderPipeline {
    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let transform_bind_group_layout = context.bind_layout_cache.get(TRANSFORM_BIND_GROUP_LAYOUT).unwrap();
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();
    let sprite_bind_group_layout = context.bind_layout_cache.get(SPRITE_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,
            transform_bind_group_layout,
            material_bind_group_layout,
            sprite_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let shader = context.device.create_shader_module(load_shader!("sprite_shader.wgsl").into());

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("sprite render pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[SmallMesh::vertex_description(), age_buffer_layout()],
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
            topology: wgpu::PrimitiveTopology::TriangleList,
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

fn age_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<f32>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32,
        }],
    }
}

pub fn render_muzzle_flashes<'a>(world: &'a World, mut render_pass: RenderPass<'a>, flash: &'a MuzzleFlash) -> RenderPass<'a> {
    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);
    render_pass.set_bind_group(1, &flash.transform_bind_group, &[]);

    render_pass.set_bind_group(2, &flash.impact_spritesheet.material.bind_group, &[]);
    render_pass.set_bind_group(3, &flash.impact_spritesheet.uniform_bind_group, &[]);

    render_pass.set_vertex_buffer(0, flash.sprite_mesh.vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, flash.age_buffer.slice(..));

    render_pass.draw(0..6, 0..(flash.sprites_age.len() as u32));

    render_pass
}

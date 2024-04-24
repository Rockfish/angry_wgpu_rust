use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::MATERIAL_BIND_GROUP_LAYOUT;
use wgpu::{RenderPass, RenderPipeline};

use crate::load_shader;
use crate::render::buffers::TRANSFORM_BIND_GROUP_LAYOUT;
use crate::render::shadow_map::{SHADOW_FILTER_BIND_GROUP_LAYOUT, ShadowMaterial};
use crate::small_mesh::SmallMesh;
use crate::world::World;

pub fn create_debug_depth_render_pipeline(context: &GpuContext) -> RenderPipeline {
    
    let shader = context.device.create_shader_module(load_shader!("debug_depth_shader.wgsl").into());

    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let transform_bind_group_layout = context.bind_layout_cache.get(TRANSFORM_BIND_GROUP_LAYOUT).unwrap();
    let shadow_filter_bind_group_layout = context.bind_layout_cache.get(SHADOW_FILTER_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            camera_bind_group_layout,
            transform_bind_group_layout,
            shadow_filter_bind_group_layout
        ],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[SmallMesh::vertex_description()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
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

pub fn create_debug_test_render_pipeline(context: &GpuContext) -> RenderPipeline {

    let shader = context.device.create_shader_module(load_shader!("debug_test_shader.wgsl").into());

    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let transform_bind_group_layout = context.bind_layout_cache.get(TRANSFORM_BIND_GROUP_LAYOUT).unwrap();
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            camera_bind_group_layout,
            transform_bind_group_layout,
            material_bind_group_layout
        ],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[SmallMesh::vertex_description()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
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

pub fn shadow_render_debug<'a>(
    mut render_pass: RenderPass<'a>,
    world: &'a World,
    shadow_map: &'a ShadowMaterial,
) -> RenderPass<'a> {
    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);
    render_pass.set_bind_group(1, &shadow_map.transform_bind_group, &[]);
    
    render_pass.set_bind_group(2, &shadow_map.filter_bind_group, &[]);
    // render_pass.set_bind_group(2, &shadow_map.test_material.bind_group, &[]);

    render_pass.set_vertex_buffer(0, shadow_map.quad_mesh.vertex_buffer.slice(..));
    render_pass.draw(0..6, 0..1);

    render_pass
}
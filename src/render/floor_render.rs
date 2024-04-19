use crate::floor::Floor;
use crate::load_shader;
use crate::params::shader_params::SHADER_PARAMETERS_BIND_GROUP_LAYOUT;
use crate::render::buffers::TRANSFORM_BIND_GROUP_LAYOUT;
use crate::render::main_render::Pipelines;
use crate::small_mesh::SmallMesh;
use crate::world::World;
use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::MATERIAL_BIND_GROUP_LAYOUT;
use wgpu::{RenderPass, RenderPipeline};

pub fn create_floor_shader_pipeline(context: &GpuContext) -> Pipelines {
    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let model_bind_group_layout = context.bind_layout_cache.get(TRANSFORM_BIND_GROUP_LAYOUT).unwrap();
    let lighting_bind_group_layout = context.bind_layout_cache.get(SHADER_PARAMETERS_BIND_GROUP_LAYOUT).unwrap();
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("floor pipeline layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,
            model_bind_group_layout,
            lighting_bind_group_layout,
            material_bind_group_layout, // diffuse
            material_bind_group_layout, // specular
            material_bind_group_layout, // emissive
        ],
        push_constant_ranges: &[],
    });

    let shader = context.device.create_shader_module(load_shader!("floor_shader.wgsl").into());

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let forward_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("floor render pipeline"),
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

    let shadow_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_shadow",
            buffers: &[SmallMesh::vertex_description()],
        },
        fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: context.device.features().contains(wgpu::Features::DEPTH_CLIP_CONTROL),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2, // corresponds to bilinear filtering
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    Pipelines {
        shadow_pipeline,
        forward_pipeline,
    }
}

pub fn render_floor<'a>(world: &'a World, mut render_pass: RenderPass<'a>, floor: &'a Floor) -> RenderPass<'a> {
    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);
    render_pass.set_bind_group(1, &floor.bind_group, &[]);
    render_pass.set_bind_group(2, &world.shader_params.bind_group, &[]);

    render_pass.set_bind_group(3, floor.material_diffuse.bind_group.as_ref(), &[]);
    render_pass.set_bind_group(4, floor.material_specular.bind_group.as_ref(), &[]);
    render_pass.set_bind_group(5, floor.material_normal.bind_group.as_ref(), &[]);

    render_pass.set_vertex_buffer(0, floor.floor_mesh.vertex_buffer.slice(..));
    render_pass.draw(0..6, 0..1);

    render_pass
}

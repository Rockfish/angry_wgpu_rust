use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::{Material, MATERIAL_BIND_GROUP_LAYOUT};
use spark_gap::model_builder::MODEL_BIND_GROUP_LAYOUT;
use spark_gap::model_mesh::ModelVertex;
use spark_gap::texture_config::TextureType;
use wgpu::{IndexFormat, RenderPass};

use crate::enemy::{EnemySystem, ENEMY_UNIFORMS_BIND_GROUP_LAYOUT};
use crate::load_shader;
use crate::params::shader_params::SHADER_PARAMETERS_BIND_GROUP_LAYOUT;
use crate::render::main_render::Pipelines;
use crate::render::textures::SHADOW_MATERIAL_BIND_GROUP_LAYOUT;
use crate::world::World;

pub fn create_enemy_shader_pipeline(context: &GpuContext) -> Pipelines {
    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let model_bind_group_layout = context.bind_layout_cache.get(MODEL_BIND_GROUP_LAYOUT).unwrap();
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();
    let params_bind_group_layout = context.bind_layout_cache.get(SHADER_PARAMETERS_BIND_GROUP_LAYOUT).unwrap();
    let instances_bind_group_layout = context.bind_layout_cache.get(ENEMY_UNIFORMS_BIND_GROUP_LAYOUT).unwrap();
    let shadow_bind_group_layout = context.bind_layout_cache.get(SHADOW_MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let shadow_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,
            model_bind_group_layout,
            params_bind_group_layout,
            instances_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let shader = context.device.create_shader_module(load_shader!("wiggle_shader.wgsl").into());

    let shadow_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow pipeline"),
        layout: Some(&shadow_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_shadow",
            buffers: &[ModelVertex::vertex_description()],
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

    let forward_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,
            model_bind_group_layout,
            params_bind_group_layout,
            instances_bind_group_layout,
            material_bind_group_layout, // diffuse
            shadow_bind_group_layout,   // shadow
        ],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let forward_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&forward_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[ModelVertex::vertex_description()],
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

    Pipelines {
        shadow_pipeline,
        forward_pipeline,
    }
}

pub fn shadow_render_enemies<'a>(context: &'a GpuContext, world: &'a World, mut render_pass: RenderPass<'a>, enemy_system: &'a EnemySystem) -> RenderPass<'a> {
    let model = &enemy_system.enemy_model;

    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);
    render_pass.set_bind_group(1, &model.bind_group, &[]);
    render_pass.set_bind_group(2, &world.shader_params.bind_group, &[]);
    render_pass.set_bind_group(3, &enemy_system.instances_bind_group, &[]);

    for mesh in model.meshes.iter() {
        model.update_mesh_buffers(context, &mesh);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

        render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..enemy_system.instances_uniforms.len() as u32);
    }

    render_pass
}

pub fn forward_render_enemies<'a>(
    context: &'a GpuContext,
    world: &'a World,
    mut render_pass: RenderPass<'a>,
    enemy_system: &'a EnemySystem,
    shadow_map: &'a Material,
) -> RenderPass<'a> {
    let model = &enemy_system.enemy_model;

    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);
    render_pass.set_bind_group(1, &model.bind_group, &[]);
    render_pass.set_bind_group(2, &world.shader_params.bind_group, &[]);
    render_pass.set_bind_group(3, &enemy_system.instances_bind_group, &[]);
    render_pass.set_bind_group(5, &shadow_map.bind_group, &[]);

    for mesh in model.meshes.iter() {
        model.update_mesh_buffers(context, &mesh);

        let diffuse_bind_group = model.get_material_bind_group(&mesh, TextureType::Diffuse);
        render_pass.set_bind_group(4, diffuse_bind_group, &[]);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

        render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..enemy_system.instances_uniforms.len() as u32);
    }

    render_pass
}

use crate::load_shader;
use crate::params::shader_params::SHADER_PARAMETERS_BIND_GROUP_LAYOUT;
use crate::world::World;
use glam::Mat4;
use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::MATERIAL_BIND_GROUP_LAYOUT;
use spark_gap::model::Model;
use spark_gap::model_builder::MODEL_BIND_GROUP_LAYOUT;
use spark_gap::model_mesh::ModelVertex;
use spark_gap::texture_config::TextureType;
use wgpu::{IndexFormat, RenderPass, RenderPipeline};

pub fn render_model<'a>(
    context: &'a GpuContext,
    world: &'a World,
    mut render_pass: RenderPass<'a>,
    model: &'a Model,
    model_transform: &Mat4,
) -> RenderPass<'a> {
    model.update_model_buffers(context, model_transform);

    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);
    render_pass.set_bind_group(1, &model.bind_group, &[]);
    render_pass.set_bind_group(2, &world.shader_params.bind_group, &[]);

    for mesh in model.meshes.iter() {
        model.update_mesh_buffers(context, &mesh);

        let diffuse_bind_group = model.get_material_bind_group(&mesh, TextureType::Diffuse);
        let specular_bind_group = model.get_material_bind_group(&mesh, TextureType::Specular);
        let emissive_bind_group = model.get_material_bind_group(&mesh, TextureType::Emissive);
        // let shadow_map_bind_group = model.get_material_bind_group(&mesh, TextureType::Diffuse); // shadow

        render_pass.set_bind_group(3, diffuse_bind_group, &[]);
        render_pass.set_bind_group(4, specular_bind_group, &[]);
        render_pass.set_bind_group(5, emissive_bind_group, &[]);
        // render_pass.set_bind_group(6, shadow_map_bind_group, &[]);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    render_pass
}

pub fn create_player_shader_pipeline(context: &GpuContext) -> RenderPipeline {
    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let model_bind_group_layout = context.bind_layout_cache.get(MODEL_BIND_GROUP_LAYOUT).unwrap();
    let params_bind_group_layout = context.bind_layout_cache.get(SHADER_PARAMETERS_BIND_GROUP_LAYOUT).unwrap();
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,
            model_bind_group_layout,
            params_bind_group_layout,
            material_bind_group_layout, // diffuse
            material_bind_group_layout, // specular
            material_bind_group_layout, // emissive
                                        // material_bind_group_layout, // shadow
        ],
        push_constant_ranges: &[],
    });

    let shader = context.device.create_shader_module(load_shader!("player_shader.wgsl").into());

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
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

    render_pipeline
}

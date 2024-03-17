use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::MATERIAL_BIND_GROUP_LAYOUT;
use spark_gap::model_builder::MODEL_BIND_GROUP_LAYOUT;
use spark_gap::model_mesh::ModelVertex;
use spark_gap::texture_config::TextureType;
use wgpu::{IndexFormat, RenderPass, RenderPipeline};

use crate::bullets::{BULLET_POSITIONS_BIND_GROUP_LAYOUT, BULLET_ROTATIONS_BIND_GROUP_LAYOUT, BulletSystem};
use crate::enemy::{EnemySystem};
use crate::load_shader;
use crate::render::buffers::instance_index_description;
use crate::small_mesh::SmallMesh;
use crate::world::World;

pub fn create_bullet_shader_pipeline(context: &GpuContext) -> RenderPipeline {
    let camera_bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();
    let bullet_positions_bind_group_layout = context.bind_layout_cache.get(BULLET_POSITIONS_BIND_GROUP_LAYOUT).unwrap();
    let bullet_rotations_bind_group_layout = context.bind_layout_cache.get(BULLET_ROTATIONS_BIND_GROUP_LAYOUT).unwrap();
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("bullet shader pipeline layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,
            bullet_positions_bind_group_layout,
            bullet_rotations_bind_group_layout,
            material_bind_group_layout, // diffuse
        ],
        push_constant_ranges: &[],
    });

    let shader = context.device.create_shader_module(load_shader!("bullet_shader.wgsl").into());

    let swapchain_capabilities = context.surface.get_capabilities(&context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bullet Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                SmallMesh::vertex_description(),
                instance_index_description(),
            ],
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

pub fn render_bullets<'a>(
    world: &'a World,
    mut render_pass: RenderPass<'a>,
    bullet_system: &'a BulletSystem,
) -> RenderPass<'a> {

    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);

    render_pass.set_bind_group(1, &bullet_system.bullet_positions_bind_group, &[]);
    render_pass.set_bind_group(2, &bullet_system.bullet_positions_bind_group, &[]);

    render_pass.set_bind_group(3, &bullet_system.bullet_material.bind_group, &[]);

    // render_pass.set_bind_group(2, &bullet_system.impact_spritesheet.material.bind_group, &[]);
    // render_pass.set_bind_group(3, &bullet_system.impact_spritesheet.uniform_bind_group, &[]);

    render_pass.set_vertex_buffer(0, bullet_system.bullet_mesh.vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, bullet_system.instances_index_buffer.slice(..));

    render_pass.draw_indexed(0..bullet_system.bullet_mesh.num_elements, 0, 0..bullet_system.instance_indexes.len() as u32);

    render_pass
}
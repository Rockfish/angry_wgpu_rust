use glam::Mat4;
use crate::world::World;
use spark_gap::camera::camera_handler::CAMERA_BIND_GROUP_LAYOUT;
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::MATERIAL_BIND_GROUP_LAYOUT;
use spark_gap::model::Model;
use spark_gap::model_builder::MODEL_BIND_GROUP_LAYOUT;
use spark_gap::model_mesh::ModelVertex;
use spark_gap::texture_config::TextureType;
use wgpu::{IndexFormat, RenderPass, RenderPassDescriptor, RenderPipeline, SurfaceTexture, TextureView};
use crate::lighting::GAME_LIGHTING_BIND_GROUP_LAYOUT;
use crate::load_shader;
use crate::player::Player;

pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

pub struct AnimRenderPass {
    player_shader_pipeline: RenderPipeline,
    pub depth_texture_view: TextureView,
}

impl AnimRenderPass {
    pub fn new(context: &GpuContext) -> Self {
        let player_shader_pipeline = create_player_shader_pipeline(context);
        let depth_texture_view = create_depth_texture_view(&context);

        Self {
            player_shader_pipeline,
            depth_texture_view,
        }
    }

    pub fn render(&mut self, context: &GpuContext, world: &World) {
        let frame = context
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                store: wgpu::StoreOp::Store,
            },
        };

        let depth_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        };

        let pass_description = wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: Some(depth_attachment),
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let player = &world.player.borrow();
        let model = player.model.borrow();

        {
            let mut render_pass = encoder.begin_render_pass(&pass_description);

            render_pass.set_pipeline(&self.player_shader_pipeline);
            let _render_pass = render_model(context, world, render_pass, &model);
        }

        context.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

fn render_model<'a>(
    context: &'a GpuContext,
    world: &'a World,
    mut render_pass: RenderPass<'a>,
    model: &'a Model,
) -> RenderPass<'a> {
    let model_transform = &world.model_transform;

    model.update_model_buffers(context, &model_transform);
    &world.game_lighting_handler.update_lighting(context);

    render_pass.set_bind_group(0, &world.camera_handler.bind_group, &[]);
    render_pass.set_bind_group(1, &model.bind_group, &[]);
    render_pass.set_bind_group(2, &world.game_lighting_handler.bind_group, &[]);

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
    let material_bind_group_layout = context.bind_layout_cache.get(MATERIAL_BIND_GROUP_LAYOUT).unwrap();
    let lighting_bind_group_layout = context.bind_layout_cache.get(GAME_LIGHTING_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,
            model_bind_group_layout,
            lighting_bind_group_layout,
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

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub fn create_depth_texture_view(context: &GpuContext) -> TextureView {
    let size = context.window.inner_size();

    let size = wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    };

    let desc = wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[DEPTH_FORMAT],
    };

    let texture = context.device.create_texture(&desc);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}


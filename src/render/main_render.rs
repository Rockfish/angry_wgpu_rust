use std::f32::consts::PI;
use glam::{Mat4, vec3, Vec3};
use crate::world::World;
use spark_gap::gpu_context::GpuContext;
use wgpu::{RenderPipeline, TextureView};
use crate::render::enemy_render::{create_enemy_shader_pipeline, render_enemy_model};
use crate::render::floor_render::{create_floor_shader_pipeline, render_floor};
use crate::render::player_render::{create_player_shader_pipeline, render_model};
use crate::render::sprite_render::create_sprite_shader_pipeline;

pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

pub struct AnimRenderPass {
    player_shader_pipeline: RenderPipeline,
    floor_shader_pipeline: RenderPipeline,
    enemy_shader_pipeline: RenderPipeline,
    sprite_shader_pipeline: RenderPipeline,
    pub depth_texture_view: TextureView,
}

impl AnimRenderPass {
    pub fn new(context: &GpuContext) -> Self {
        let player_shader_pipeline = create_player_shader_pipeline(context);
        let floor_shader_pipeline = create_floor_shader_pipeline(context);
        let enemy_shader_pipeline = create_enemy_shader_pipeline(context);
        let sprite_shader_pipeline = create_sprite_shader_pipeline(context);

        let depth_texture_view = create_depth_texture_view(&context);

        Self {
            player_shader_pipeline,
            floor_shader_pipeline,
            enemy_shader_pipeline,
            sprite_shader_pipeline,
            depth_texture_view,
        }
    }

    pub fn resize(&mut self, context: &GpuContext) {
        self.depth_texture_view = create_depth_texture_view(context);
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
        let enemy_system = &world.enemy_system.borrow();
        let enemy_model = &enemy_system.enemy_model;

        world.shader_params.update_buffer(context);

        let floor = &world.floor.borrow();

        {
            let mut render_pass = encoder.begin_render_pass(&pass_description);

            // floor
            render_pass.set_pipeline(&self.floor_shader_pipeline);
            render_pass = render_floor(context, world, render_pass, floor);

            // player
            render_pass.set_pipeline(&self.player_shader_pipeline);
            render_pass = render_model(context, world, render_pass, &model, &world.player_transform);

            // enemy
            let mut model_transform = world.player_transform * Mat4::from_scale(vec3(8.0, 8.0, 8.0));
            // model_transform *= Mat4::from_scale(Vec3::splat(0.01));
            // model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monster_theta);
            model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
            model_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

            render_pass.set_pipeline(&self.enemy_shader_pipeline);
            render_pass = render_enemy_model(context, world, render_pass, &enemy_model, &model_transform);
        }

        context.queue.submit(Some(encoder.finish()));
        frame.present();
    }
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


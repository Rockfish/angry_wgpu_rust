use spark_gap::gpu_context::GpuContext;
use wgpu::{CommandEncoder, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, TextureView};

use crate::render::bullet_render::{create_bullet_shader_pipeline, render_bullets};
use crate::render::debug_render::{create_debug_depth_render_pipeline, shadow_render_debug};
use crate::render::enemy_render::{create_enemy_shader_pipeline, forward_render_enemies, shadow_render_enemies};
use crate::render::floor_render::{create_floor_shader_pipeline, forward_render_floor, shadow_render_floor};
use crate::render::player_render::{create_player_shader_pipeline, forward_render_player, shadow_render_player};
use crate::render::shadow_map::{create_shadow_map_material, ShadowMaterial};
use crate::render::sprite_render::{create_sprite_shader_pipeline, render_muzzle_flashes};
use crate::render::textures::create_depth_texture_view;
use crate::world::World;

pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

pub struct Pipelines {
    pub(crate) shadow_pipeline: RenderPipeline,
    pub(crate) forward_pipeline: RenderPipeline,
}

pub struct WorldRender {
    player_shader_pipelines: Pipelines,
    floor_shader_pipelines: Pipelines,
    enemy_shader_pipelines: Pipelines,
    sprite_shader_pipeline: RenderPipeline,
    bullet_shader_pipeline: RenderPipeline,
    depth_debug_pipeline: RenderPipeline,
    pub depth_texture_view: TextureView,
    shadow_map_material: ShadowMaterial,
}

impl WorldRender {
    pub fn new(context: &mut GpuContext) -> Self {
        let depth_texture_view = create_depth_texture_view(&context);

        let shadow_map_material = create_shadow_map_material(context);

        let player_shader_pipelines = create_player_shader_pipeline(context);
        let floor_shader_pipelines = create_floor_shader_pipeline(context);
        let enemy_shader_pipelines = create_enemy_shader_pipeline(context);
        let sprite_shader_pipeline = create_sprite_shader_pipeline(context);
        let bullet_shader_pipeline = create_bullet_shader_pipeline(context);
        
        let depth_debug_pipeline = create_debug_depth_render_pipeline(context);

        Self {
            player_shader_pipelines,
            floor_shader_pipelines,
            enemy_shader_pipelines,
            sprite_shader_pipeline,
            bullet_shader_pipeline,
            depth_debug_pipeline,
            depth_texture_view,
            shadow_map_material,
        }
    }

    pub fn resize(&mut self, context: &GpuContext) {
        self.depth_texture_view = create_depth_texture_view(context);
    }

    pub fn render(&mut self, context: &GpuContext, world: &World) {
        world.shader_params.update_buffer(context);

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let frame = context.surface.get_current_texture().expect("Failed to acquire next swap chain texture");
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // shadow pass
        {
            let depth_stencil_attachment = RenderPassDepthStencilAttachment {
                view: &self.shadow_map_material.texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            };

            let shadow_pass_descriptor = RenderPassDescriptor {
                label: Some("shadow render pass descriptor"),
                color_attachments: &[],
                depth_stencil_attachment: Some(depth_stencil_attachment),
                timestamp_writes: None,
                occlusion_query_set: None,
            };

            self.shadow_render_pass(context, world, &mut encoder, &shadow_pass_descriptor);
        }

        // forward pass
        {
            let depth_attachment = RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            };
        
            let color_attachment = RenderPassColorAttachment {
                view: &frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            };
        
            let forward_pass_description = RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(depth_attachment),
                timestamp_writes: None,
                occlusion_query_set: None,
            };

            self.main_render_pass(context, world, &mut encoder, &forward_pass_description);
            
            self.debug_render_pass(world, &mut encoder, &forward_pass_description);
        }

        context.queue.submit(Some(encoder.finish()));
        frame.present();
    }
    
    fn debug_render_pass(&self, world: &World, encoder: &mut CommandEncoder, pass_description: &RenderPassDescriptor) {
        let floor = &world.floor.borrow();

        let mut render_pass = encoder.begin_render_pass(pass_description);
        
        render_pass.set_pipeline(&self.depth_debug_pipeline);
        render_pass = shadow_render_debug(render_pass, world, &self.shadow_map_material, floor);
    }

    fn shadow_render_pass(&self, context: &GpuContext, world: &World, encoder: &mut CommandEncoder, pass_description: &RenderPassDescriptor) {
        let floor = &world.floor.borrow();
        let player = &world.player.borrow();
        let enemy_system = &world.enemy_system.borrow();

        let mut render_pass = encoder.begin_render_pass(pass_description);

        // floor
        render_pass.set_pipeline(&self.floor_shader_pipelines.shadow_pipeline);
        render_pass = shadow_render_floor(world, render_pass, floor);

        // player
        render_pass.set_pipeline(&self.player_shader_pipelines.shadow_pipeline);
        render_pass = shadow_render_player(context, world, render_pass, player);

        // enemies
        render_pass.set_pipeline(&self.enemy_shader_pipelines.shadow_pipeline);
        render_pass = shadow_render_enemies(context, world, render_pass, enemy_system);
    }

    fn main_render_pass(&self, context: &GpuContext, world: &World, encoder: &mut CommandEncoder, pass_description: &RenderPassDescriptor) {
        let floor = &world.floor.borrow();
        let player = &world.player.borrow();
        let flashes = &world.muzzle_flash.borrow();
        let enemy_system = &world.enemy_system.borrow();
        let bullet_system = &world.bullet_system.borrow();

        let mut render_pass = encoder.begin_render_pass(pass_description);

        // floor
        render_pass.set_pipeline(&self.floor_shader_pipelines.forward_pipeline);
        render_pass = forward_render_floor(world, render_pass, floor, &self.shadow_map_material);

        // player
        render_pass.set_pipeline(&self.player_shader_pipelines.forward_pipeline);
        render_pass = forward_render_player(context, world, render_pass, player, &self.shadow_map_material);

        // muzzle flashes
        render_pass.set_pipeline(&self.sprite_shader_pipeline);
        render_pass = render_muzzle_flashes(world, render_pass, flashes);

        // bullets
        render_pass.set_pipeline(&self.bullet_shader_pipeline);
        render_pass = render_bullets(world, render_pass, bullet_system);

        // enemies
        render_pass.set_pipeline(&self.enemy_shader_pipelines.forward_pipeline);
        render_pass = forward_render_enemies(context, world, render_pass, enemy_system, &self.shadow_map_material);
    }
}

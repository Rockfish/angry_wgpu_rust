use std::rc::Rc;

use spark_gap::gpu_context::GpuContext;
use spark_gap::material::Material;
use spark_gap::texture_config::TextureType;
use wgpu::{BindGroup, BindGroupLayout, RenderPipeline, Sampler, TextureView};

use crate::render::bullet_render::{create_bullet_shader_pipeline, render_bullets};
use crate::render::enemy_render::{create_enemy_shader_pipeline, render_enemy_model};
use crate::render::floor_render::{create_floor_shader_pipeline, render_floor};
use crate::render::player_render::{create_player_shader_pipeline, render_player};
use crate::render::sprite_render::{create_sprite_shader_pipeline, render_muzzle_flashes};
use crate::world::World;

pub const SHADOW_WIDTH: u32 = 6 * 1024;
pub const SHADOW_HEIGHT: u32 = 6 * 1024;

pub const SHADOW_MATERIAL_BIND_GROUP_LAYOUT: &str = "shadow material bind group layout";

pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

pub struct WorldRender {
    player_shader_pipeline: RenderPipeline,
    floor_shader_pipeline: RenderPipeline,
    enemy_shader_pipeline: RenderPipeline,
    sprite_shader_pipeline: RenderPipeline,
    bullet_shader_pipeline: RenderPipeline,
    pub depth_texture_view: TextureView,
    shadow_map_material: Material,
}

impl WorldRender {
    pub fn new(context: &mut GpuContext) -> Self {
        let player_shader_pipeline = create_player_shader_pipeline(context);
        let floor_shader_pipeline = create_floor_shader_pipeline(context);
        let enemy_shader_pipeline = create_enemy_shader_pipeline(context);
        let sprite_shader_pipeline = create_sprite_shader_pipeline(context);
        let bullet_shader_pipeline = create_bullet_shader_pipeline(context);

        let depth_texture_view = create_depth_texture_view(&context);
        let shadow_map_material = create_shadow_map_material(context);

        Self {
            player_shader_pipeline,
            floor_shader_pipeline,
            enemy_shader_pipeline,
            sprite_shader_pipeline,
            bullet_shader_pipeline,
            depth_texture_view,
            shadow_map_material,
        }
    }

    pub fn resize(&mut self, context: &GpuContext) {
        self.depth_texture_view = create_depth_texture_view(context);
    }

    pub fn render(&mut self, context: &GpuContext, world: &World) {
        let frame = context.surface.get_current_texture().expect("Failed to acquire next swap chain texture");

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

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        world.shader_params.update_buffer(context);

        let floor = &world.floor.borrow();
        let player = &world.player.borrow();
        let flashes = &world.muzzle_flash.borrow();
        let enemy_system = &world.enemy_system.borrow();
        let bullet_system = &world.bullet_system.borrow();

        {
            let mut render_pass = encoder.begin_render_pass(&pass_description);

            // floor
            render_pass.set_pipeline(&self.floor_shader_pipeline);
            render_pass = render_floor(world, render_pass, floor);

            // player
            render_pass.set_pipeline(&self.player_shader_pipeline);
            render_pass = render_player(context, world, render_pass, player);

            // muzzle flashes
            render_pass.set_pipeline(&self.sprite_shader_pipeline);
            render_pass = render_muzzle_flashes(world, render_pass, flashes);

            // bullets
            render_pass.set_pipeline(&self.bullet_shader_pipeline);
            render_pass = render_bullets(world, render_pass, bullet_system);

            // enemies
            render_pass.set_pipeline(&self.enemy_shader_pipeline);
            render_pass = render_enemy_model(context, world, render_pass, enemy_system);
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

pub fn create_shadow_map_material(context: &mut GpuContext) -> Material {
    
    let shadow_map_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: SHADOW_WIDTH,
            height: SHADOW_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float, // A common format for depth textures
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: Some("Shadow Map Texture"),
        view_formats: &[DEPTH_FORMAT],
    });

    let shadow_map_view = shadow_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let shadow_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        compare: Some(wgpu::CompareFunction::LessEqual), // Enable depth comparison
        ..Default::default()
    });

    if !context.bind_layout_cache.contains_key(SHADOW_MATERIAL_BIND_GROUP_LAYOUT) {
        let layout = create_shadow_material_bind_group_layout(context);
        context
            .bind_layout_cache
            .insert(String::from(SHADOW_MATERIAL_BIND_GROUP_LAYOUT), layout.into());
    }

    let bind_group_layout = context.bind_layout_cache.get(SHADOW_MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let bind_group = create_shadow_texture_bind_group(context, &bind_group_layout, &shadow_map_view, &shadow_sampler);

    Material {
        texture_path: Default::default(),
        texture_type: TextureType::None,
        texture: Rc::new(shadow_map_texture),
        view: Rc::new(shadow_map_view),
        sampler: Rc::new(shadow_sampler),
        bind_group: Rc::new(bind_group),
        width: SHADOW_WIDTH,
        height: SHADOW_HEIGHT,
    }
}

pub fn create_shadow_material_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            // 0: texture
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Depth,
                },
                count: None,
            },
            // 1: sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
        label: Some(SHADOW_MATERIAL_BIND_GROUP_LAYOUT),
    })
}

pub fn create_shadow_texture_bind_group(
    context: &GpuContext,
    bind_group_layout: &BindGroupLayout,
    texture_view: &TextureView,
    texture_sampler: &Sampler,
) -> BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(texture_sampler),
            },
        ],
        label: Some("shadow material bind group"),
    })
}

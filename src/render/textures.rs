use std::rc::Rc;

use spark_gap::gpu_context::GpuContext;
use spark_gap::material::Material;
use spark_gap::texture_config::TextureType;
use wgpu::{BindGroupLayout, Texture, TextureView};

pub const SHADOW_WIDTH: u32 = 6 * 1024;
pub const SHADOW_HEIGHT: u32 = 6 * 1024;

pub const SHADOW_MATERIAL_BIND_GROUP_LAYOUT: &str = "shadow material bind group layout";

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

pub fn create_shadow_texture_view(shadow_texture: &Texture, layer_id: u32) -> TextureView {
    shadow_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(&format!("shadow id: {}", layer_id)),
        format: None,
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: layer_id,
        array_layer_count: Some(1),
    })
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
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: Some("shadow map texture"),
        view_formats: &[],
    });

    // does which is used matter?
    // let shadow_view = shadow_map_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let shadow_view = create_shadow_texture_view(&shadow_map_texture, 0);

    let shadow_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    if !context.bind_layout_cache.contains_key(SHADOW_MATERIAL_BIND_GROUP_LAYOUT) {
        let layout = create_shadow_material_bind_group_layout(context);
        context.bind_layout_cache.insert(String::from(SHADOW_MATERIAL_BIND_GROUP_LAYOUT), layout.into());
    }

    let bind_group_layout = context.bind_layout_cache.get(SHADOW_MATERIAL_BIND_GROUP_LAYOUT).unwrap();

    let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&shadow_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&shadow_sampler),
            },
        ],
        label: Some("shadow material bind group"),
    });

    Material {
        texture_path: Default::default(),
        texture_type: TextureType::None,
        texture: Rc::new(shadow_map_texture),
        view: Rc::new(shadow_view),
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
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
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

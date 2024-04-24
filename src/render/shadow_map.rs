use glam::{Mat4, vec3};
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::Material;
use spark_gap::texture_config::{TextureConfig, TextureWrap};
use wgpu::{BindGroup, BindGroupLayout, Sampler, Texture, TextureView};
use crate::quads::create_unit_square;
use crate::render::buffers::{create_buffer_bind_group, create_mat4_buffer_init, create_uniform_bind_group_layout, get_or_create_bind_group_layout, TRANSFORM_BIND_GROUP_LAYOUT};
use crate::small_mesh::SmallMesh;

pub const SHADOW_WIDTH: u32 = 6 * 1024;
pub const SHADOW_HEIGHT: u32 = 6 * 1024;

pub const SHADOW_BIND_GROUP_LAYOUT: &str = "shadow comparison bind group layout";
pub const SHADOW_COMPARISON_BIND_GROUP_LAYOUT: &str = "shadow comparison bind group layout";
pub const SHADOW_FILTER_BIND_GROUP_LAYOUT: &str = "shadow filter bind group layout";



pub struct ShadowMaterial {
    pub quad_mesh: SmallMesh,
    pub test_material: Material,
    pub texture: Texture,
    pub texture_view: TextureView,
    pub texture_sampler: Sampler,
    pub filter_bind_group: BindGroup,
    pub comparison_sampler: Sampler,
    pub comparison_bind_group: BindGroup,
    pub transform_bind_group: BindGroup,
}


pub fn create_shadow_map_material(context: &mut GpuContext) -> ShadowMaterial {

    let texture = context.device.create_texture(&wgpu::TextureDescriptor {
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

    // let texture_view = create_shadow_texture_view(&texture, 0);
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let texture_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    if !context.bind_layout_cache.contains_key(SHADOW_FILTER_BIND_GROUP_LAYOUT) {
        let layout = create_shadow_filter_bind_group_layout(context);
        context
            .bind_layout_cache
            .insert(String::from(SHADOW_FILTER_BIND_GROUP_LAYOUT), layout.into());
    }

    let shadow_filter_layout = context.bind_layout_cache.get(SHADOW_FILTER_BIND_GROUP_LAYOUT).unwrap();

    let filter_bind_group = create_texture_bind_group(context, &shadow_filter_layout, &texture_view, &texture_sampler);

    // comparison 
    
    let comparison_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
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

    if !context.bind_layout_cache.contains_key(SHADOW_COMPARISON_BIND_GROUP_LAYOUT) {
        let layout = create_shadow_comparison_bind_group_layout(context);
        context.bind_layout_cache.insert(String::from(SHADOW_COMPARISON_BIND_GROUP_LAYOUT), layout.into());
    }

    let shadow_bind_group_layout = context.bind_layout_cache.get(SHADOW_COMPARISON_BIND_GROUP_LAYOUT).unwrap();

    let comparison_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: shadow_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&comparison_sampler),
            },
        ],
        label: Some("shadow material bind group"),
    });

    let scale = 1.0f32;
    let mut model_transform = Mat4::from_scale(vec3(scale, scale, scale));
    model_transform *= Mat4::from_rotation_x(-90.0f32.to_radians());

    let transform_buffer = create_mat4_buffer_init(context, &model_transform, "shadow debug transform");
    let layout = get_or_create_bind_group_layout(context, TRANSFORM_BIND_GROUP_LAYOUT, create_uniform_bind_group_layout);
    let transform_bind_group = create_buffer_bind_group(context, &layout, &transform_buffer, "shadow debug transform bind group");

    let quad_mesh = create_unit_square(context);

    // test 
    let texture_config = TextureConfig::new().set_wrap(TextureWrap::Clamp);
    let test_material = Material::new(context, "angrygl_assets/bullet/red_and_green_bullet_transparent.png", &texture_config).unwrap();

    ShadowMaterial {
        quad_mesh,
        test_material,
        texture,
        texture_view,
        texture_sampler,
        filter_bind_group,
        comparison_sampler,
        comparison_bind_group,
        transform_bind_group,
    }
}

fn create_shadow_filter_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
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
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
        label: Some(SHADOW_FILTER_BIND_GROUP_LAYOUT),
    })
}

pub fn create_shadow_comparison_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
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
        label: Some(SHADOW_COMPARISON_BIND_GROUP_LAYOUT),
    })
}

fn create_shadow_texture_view(shadow_texture: &Texture, layer_id: u32) -> TextureView {
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

fn create_texture_bind_group(
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
        label: Some("shadow filter bind group"),
    })
}

use std::mem;

use glam::{Mat4, vec3};
use spark_gap::buffers::create_mat4_buffer_init;
use spark_gap::gpu_context::{get_or_create_bind_group_layout, GpuContext};
use spark_gap::small_mesh::{create_unit_square, SmallMesh};
use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPass, RenderPipeline, Sampler, Texture, TextureView};
use wgpu::util::DeviceExt;

use crate::load_shader;

pub const SHADOW_WIDTH: u32 = 6 * 1024;
pub const SHADOW_HEIGHT: u32 = 6 * 1024;
pub const SHADOW_TEXTURE_LAYERS: u32 = 2;

pub const SHADOW_USE_BIND_GROUP_LAYOUT: &str = "shadow use bind group layout";
pub const SHADOW_DEBUG_BIND_GROUP_LAYOUT: &str = "shadow debug bind group layout";

// Shadow texture, filter sampler, and buffers for debug shader
pub struct ShadowMaterial {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub texture_sampler: Sampler,
    pub stencil_views: Vec<TextureView>,
    pub quad_mesh: SmallMesh,
    pub projection_view_buffer: Buffer,
    pub transform_buffer: Buffer,
    pub layer_num_buffer: Buffer,
    pub shadow_use_bind_group: BindGroup,
    pub shadow_debug_bind_group: BindGroup,
    pub shadow_debug_pipeline: RenderPipeline,
}

pub fn create_shadow_map_material(context: &mut GpuContext) -> ShadowMaterial {
    let quad_mesh = create_unit_square(context);

    let scale = 400.0f32;
    let mut model_transform = Mat4::from_scale(vec3(scale, scale, scale));
    model_transform *= Mat4::from_rotation_z(180.0f32.to_radians());

    let projection_view_buffer = create_mat4_buffer_init(context, &model_transform, "shadow debug projection view");

    let transform_buffer = create_mat4_buffer_init(context, &model_transform, "shadow debug transform");

    let layer_num = 0_u32;

    let layer_num_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("layer number"),
        contents: bytemuck::bytes_of(&layer_num),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let texture_size = wgpu::Extent3d {
        width: 2048,
        height: 2048,
        depth_or_array_layers: 2,
    };

    // multi layered depth texture
    let texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: Some("shadow map texture"),
        view_formats: &[],
    });

    // this view is for using the texture
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // set of per layer views to attach as depth stencil to write to the individual texture layer 
    // mostly one per light
    let stencil_views = (0..SHADOW_TEXTURE_LAYERS).map(|i| {
        create_shadow_texture_view(&texture, i)
    }).collect::<Vec<TextureView>>();

    let texture_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let shadow_use_layout =
        get_or_create_bind_group_layout(context, SHADOW_USE_BIND_GROUP_LAYOUT, create_shadow_use_bind_group_layout);

    let shadow_use_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("shadow filter bind group"),
        layout: &shadow_use_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture_sampler),
            },
        ],
    });

    let shadow_debug_layout =
        get_or_create_bind_group_layout(context, SHADOW_DEBUG_BIND_GROUP_LAYOUT, create_shadow_debug_bind_group_layout);

    let shadow_debug_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("shadow filter bind group"),
        layout: &shadow_debug_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_view_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: transform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: layer_num_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&texture_sampler),
            },
        ],
    });

    let shadow_debug_pipeline = create_debug_depth_render_pipeline(context);

    ShadowMaterial {
        texture,
        texture_view,
        texture_sampler,
        stencil_views,
        quad_mesh,
        projection_view_buffer,
        transform_buffer,
        layer_num_buffer,
        shadow_use_bind_group,
        shadow_debug_bind_group,
        shadow_debug_pipeline,
    }
}

fn create_shadow_use_bind_group_layout(context: &GpuContext, label: &str) -> BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            // texture
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            // sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
        label: Some(label),
    })
}

fn create_shadow_debug_bind_group_layout(context: &GpuContext, label: &str) -> BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            // projection_view
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(mem::size_of::<Mat4>() as _),
                },
                count: None,
            },
            // transform
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(mem::size_of::<Mat4>() as _),
                },
                count: None,
            },
            // layer number
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(mem::size_of::<u32>() as _),
                },
                count: None,
            },
            // texture
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            // sampler
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
        label: Some(label),
    })
}

pub fn create_debug_depth_render_pipeline(gpu_context: &GpuContext) -> RenderPipeline {

    let shader = gpu_context.device.create_shader_module(load_shader!("debug_shadow_shader.wgsl").into());

    let shadow_debug_bind_group_layout = gpu_context.bind_layout_cache.get(SHADOW_DEBUG_BIND_GROUP_LAYOUT).unwrap();

    let pipeline_layout = gpu_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[shadow_debug_bind_group_layout],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = gpu_context.surface.get_capabilities(&gpu_context.adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = gpu_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("debug texture pipeline"),
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
        primitive: wgpu::PrimitiveState::default(),
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

pub fn shadow_render_debug<'a>(mut render_pass: RenderPass<'a>, shadow_map: &'a ShadowMaterial) -> RenderPass<'a> {
    render_pass.set_bind_group(0, &shadow_map.shadow_debug_bind_group, &[]);

    render_pass.set_vertex_buffer(0, shadow_map.quad_mesh.vertex_buffer.slice(..));
    render_pass.draw(0..6, 0..1);

    render_pass
}

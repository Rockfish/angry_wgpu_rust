use crate::params::common::{DirectionLight, PointLight};
use glam::{vec4, Mat4, Vec3, Vec4};
use spark_gap::gpu_context::GpuContext;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

pub const SHADER_PARAMETERS_BIND_GROUP_LAYOUT: &str = "shader_params_bind_group_layout";

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderParametersUniform {
    pub direction_light: DirectionLight,
    pub point_light: PointLight,
    pub model_rotation: Mat4,
    pub light_space_matrix: Mat4,
    pub view_position: Vec4,
    pub ambient_color: Vec4,
    pub time: f32,
    pub depth_mode: i32,
    pub use_light: i32,
    pub use_point_light: i32,
    pub use_emissive: i32,
    pub use_specular: i32,
    pub _pad: [f32; 2],
}

pub struct ShaderParametersHandler {
    pub uniform: ShaderParametersUniform,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
}

impl ShaderParametersHandler {
    pub fn new(context: &mut GpuContext) -> Self {
        let uniform = ShaderParametersUniform {
            direction_light: Default::default(),
            point_light: Default::default(),
            model_rotation: Default::default(),
            light_space_matrix: Default::default(),
            view_position: Default::default(),
            ambient_color: Default::default(),
            time: 0.0,
            depth_mode: 0,
            use_light: 1,
            use_point_light: 0,
            use_emissive: 0,
            use_specular: 0,
            _pad: [0.0; 2],
        };

        let uniform_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Game params buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        if !context.bind_layout_cache.contains_key(SHADER_PARAMETERS_BIND_GROUP_LAYOUT) {
            let layout = create_game_params_bind_group_layout(context);
            context
                .bind_layout_cache
                .insert(String::from(SHADER_PARAMETERS_BIND_GROUP_LAYOUT), layout.into());
        }

        let bind_group_layout = context.bind_layout_cache.get(SHADER_PARAMETERS_BIND_GROUP_LAYOUT).unwrap();

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("game_params_bind_group"),
        });

        Self {
            uniform,
            uniform_buffer,
            bind_group,
        }
    }

    pub fn update_buffer(&self, context: &GpuContext) {
        // println!("uniform: {:#?}", &self.uniform);
        context.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn set_model_rotation(&mut self, val: Mat4) {
        self.uniform.model_rotation = val;
    }

    pub fn set_light_space_matrix(&mut self, val: Mat4) {
        self.uniform.light_space_matrix = val;
    }

    pub fn set_direction_light_direction(&mut self, val: Vec3) {
        self.uniform.direction_light.direction = vec4(val.x, val.y, val.z, 1.0);
    }

    pub fn set_direction_light_color(&mut self, val: Vec3) {
        self.uniform.direction_light.color = vec4(val.x, val.y, val.z, 1.0);
    }

    pub fn set_point_light_position(&mut self, val: Vec3) {
        self.uniform.point_light.world_pos = vec4(val.x, val.y, val.z, 1.0);
    }

    pub fn set_point_light_color(&mut self, val: Vec3) {
        self.uniform.point_light.color = vec4(val.x, val.y, val.z, 1.0);
    }

    pub fn set_view_position(&mut self, val: Vec3) {
        self.uniform.view_position = vec4(val.x, val.y, val.z, 1.0);
    }

    pub fn set_ambient_color(&mut self, val: Vec3) {
        self.uniform.ambient_color = vec4(val.x, val.y, val.z, 1.0);
    }

    pub fn set_depth_mode(&mut self, val: bool) {
        self.uniform.depth_mode = if val { 1 } else { 0 };
    }

    pub fn set_use_light(&mut self, val: bool) {
        self.uniform.use_light = if val { 1 } else { 0 };
    }

    pub fn set_use_point_light(&mut self, val: bool) {
        self.uniform.use_point_light = if val { 1 } else { 0 };
    }

    pub fn set_use_emissive(&mut self, val: bool) {
        self.uniform.use_emissive = if val { 1 } else { 0 };
    }

    pub fn set_use_specular(&mut self, val: bool) {
        self.uniform.use_specular = if val { 1 } else { 0 };
    }

    pub fn set_time(&mut self, val: f32) {
        self.uniform.time = val;
    }
}

fn create_game_params_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(SHADER_PARAMETERS_BIND_GROUP_LAYOUT),
    })
}

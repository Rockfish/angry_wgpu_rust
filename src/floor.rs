use crate::render::buffers::{
    create_buffer_bind_group, create_mat4_buffer_init, create_uniform_bind_group_layout, get_or_create_bind_group_layout, TRANSFORM_BIND_GROUP_LAYOUT,
};
use crate::small_mesh::SmallMesh;
use glam::{vec3, Mat4, Vec3};
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::Material;
use spark_gap::texture_config::{TextureConfig, TextureFilter, TextureType, TextureWrap};
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, Buffer};

const FLOOR_SIZE: f32 = 100.0;
const TILE_SIZE: f32 = 1.0;
const NUM_TILE_WRAPS: f32 = FLOOR_SIZE / TILE_SIZE;

#[rustfmt::skip]
const FLOOR_VERTICES: [f32; 30] = [
    // Vertices                                // TexCoord
    -FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, 0.0,
    -FLOOR_SIZE / 2.0, 0.0,  FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, 0.0,
     FLOOR_SIZE / 2.0, 0.0,  FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, NUM_TILE_WRAPS,
    -FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, 0.0,
     FLOOR_SIZE / 2.0, 0.0,  FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, NUM_TILE_WRAPS,
     FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, NUM_TILE_WRAPS
];

pub struct Floor {
    pub floor_mesh: SmallMesh,
    pub material_diffuse: Material,
    pub material_normal: Material,
    pub material_specular: Material,
    pub model_transform: Mat4,
    pub transform_buffer: Buffer,
    pub transform_bind_group: BindGroup,
}

impl Floor {
    pub fn new(context: &mut GpuContext) -> Self {
        let texture_config = TextureConfig {
            flip_v: false,
            flip_h: false,
            gamma_correction: false,
            filter: TextureFilter::Linear,
            texture_type: TextureType::None,
            wrap: TextureWrap::Repeat,
        };

        let material_diffuse = Material::new(context, "assets/Models/Floor D.png", &texture_config).unwrap();
        let material_normal = Material::new(context, "assets/Models/Floor N.png", &texture_config).unwrap();
        let material_specular = Material::new(context, "assets/Models/Floor M.png", &texture_config).unwrap();

        let vertex_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&FLOOR_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let floor_mesh = SmallMesh {
            vertex_buffer: vertex_buffer.into(),
            num_elements: 6,
        };

        let model_transform = Mat4::IDENTITY;

        let transform_buffer = create_mat4_buffer_init(context, &model_transform, "floor transform");
        let layout = get_or_create_bind_group_layout(context, TRANSFORM_BIND_GROUP_LAYOUT, create_uniform_bind_group_layout);
        let transform_bind_group = create_buffer_bind_group(context, &layout, &transform_buffer, "floor transform bind");

        Self {
            floor_mesh,
            material_diffuse,
            material_normal,
            material_specular,
            model_transform,
            transform_buffer,
            transform_bind_group,
        }
    }

    pub fn draw(&self, context: &GpuContext, projection_view: &Mat4) {
        // shader.use_shader();

        // bind_texture(shader, 0, "texture_diffuse", &self.texture_floor_diffuse);
        // bind_texture(shader, 1, "texture_normal", &self.texture_floor_normal);
        // bind_texture(shader, 2, "texture_spec", &self.texture_floor_spec);

        // angle floor
        let _model = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), 45.0f32.to_radians());

        let model = Mat4::IDENTITY;

        // shader.set_mat4("PV", projection_view);
        // shader.set_mat4("model", &model);

        // unsafe {
        //     gl::BindVertexArray(self.floor_vao);
        //     gl::DrawArrays(gl::TRIANGLES, 0, 6);
        //     gl::BindVertexArray(0);
        // }
    }
}

use crate::small_mesh::SmallMesh;
use glam::{vec3, Mat4, Vec3};
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::Material;
use spark_gap::texture_config::{TextureConfig, TextureWrap};

const BURN_MARK_TIME: f32 = 5.0;

pub struct BurnMark {
    position: Vec3,
    time_left: f32,
}

pub struct BurnMarks {
    unit_square: SmallMesh,
    mark_material: Material,
    marks: Vec<BurnMark>,
}

impl BurnMarks {
    pub fn new(context: &mut GpuContext, unit_square: SmallMesh) -> Self {
        let texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);
        let mark_material = Material::new(context, "angrygl_assets/bullet/burn_mark.png", &texture_config).unwrap();

        Self {
            unit_square,
            mark_material,
            marks: vec![],
        }
    }

    pub fn add_mark(&mut self, position: Vec3) {
        self.marks.push(BurnMark {
            position,
            time_left: BURN_MARK_TIME,
        });
    }

    pub fn draw_marks(&mut self, projection_view: &Mat4, delta_time: f32) {
        if self.marks.is_empty() {
            return;
        }

        // shader.use_shader();
        // shader.set_mat4("PV", projection_view);

        // bind_texture(shader, 0, "texture_diffuse", &self.mark_texture);
        // bind_texture(shader, 1, "texture_normal", &self.mark_texture);

        // unsafe {
        //     gl::Enable(gl::BLEND);
        //     gl::DepthMask(gl::FALSE);
        //     gl::Disable(gl::CULL_FACE);
        //     gl::BindVertexArray(self.unit_square_vao as GLuint);
        // }

        for mark in self.marks.iter_mut() {
            let scale: f32 = 0.5 * mark.time_left;
            mark.time_left -= delta_time;

            // model *= Mat4::from_translation(vec3(mark.x, 0.01, mark.z));
            let mut model = Mat4::from_translation(mark.position);

            model *= Mat4::from_rotation_x(-90.0f32.to_radians());
            model *= Mat4::from_scale(vec3(scale, scale, scale));

            // shader.set_mat4("model", &model);

            // unsafe {
            //     gl::DrawArrays(gl::TRIANGLES, 0, 6);
            // }
        }

        self.marks.retain(|m| m.time_left > 0.0);

        // unsafe {
        //     gl::Disable(gl::BLEND);
        //     gl::DepthMask(gl::TRUE);
        //     gl::Enable(gl::CULL_FACE);
        // }
    }
}

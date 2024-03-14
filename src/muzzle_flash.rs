use glam::{Mat4, vec3};
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::Material;
use spark_gap::texture_config::{TextureConfig, TextureWrap};
use wgpu::{BindGroup, Buffer};

use crate::render::buffers::{create_buffer_bind_group, create_mat4_buffer, create_uniform_bind_group_layout, create_vertex_buffer, get_or_create_bind_group_layout, TRANSFORM_BIND_GROUP_LAYOUT, update_mat4_buffer, update_uniform_buffer};
use crate::small_mesh::SmallMesh;
use crate::sprite_sheet::SpriteSheet;

const MAX_FLASHES: usize = 50;

pub struct MuzzleFlash {
    pub sprite_mesh: SmallMesh,
    pub impact_spritesheet: SpriteSheet,
    pub sprites_age: Vec<f32>,
    pub age_buffer: Buffer,
    pub transform_buffer: Buffer,
    pub transform_bind_group: BindGroup
}

impl MuzzleFlash {
    pub fn new(context: &mut GpuContext, unit_square: SmallMesh) -> Self {
        let texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);
        let muzzle_flash_material = Material::new(context, "angrygl_assets/Player/muzzle_spritesheet.png", &texture_config).unwrap();
        let muzzle_flash_impact_spritesheet = SpriteSheet::new(context, muzzle_flash_material, 6, 0.03);

        let mut age_vec = vec![0.0_f32; MAX_FLASHES];
        let age_buffer = create_vertex_buffer(context, age_vec.as_slice(), "sprite age vec");
        age_vec.clear();

        let transform_buffer = create_mat4_buffer(context, &Mat4::IDENTITY, "muzzle flash transform");
        let layout = get_or_create_bind_group_layout(context, TRANSFORM_BIND_GROUP_LAYOUT, create_uniform_bind_group_layout);
        let bind_group = create_buffer_bind_group(context, &layout, &transform_buffer, "muzzle flash transform bind");

        Self {
            sprite_mesh: unit_square,
            impact_spritesheet: muzzle_flash_impact_spritesheet,
            sprites_age: age_vec,
            age_buffer,
            transform_buffer,
            transform_bind_group: bind_group,
        }
    }

    // pub fn update(&mut self, delta_time: f32) {
    //     if !self.sprites_age.is_empty() {
    //         for i in 0..self.sprites_age.len() {
    //             self.sprites_age[i] += delta_time;
    //         }
    //         let max_age = self.impact_spritesheet.uniform.num_columns * self.impact_spritesheet.uniform.time_per_sprite;
    //
    //         self.sprites_age.retain(|age| *age < max_age);
    //     }
    //
    //     update_uniform_buffer(context, )
    // }

    pub fn update(&mut self, context: &GpuContext, delta_time: f32, muzzle_transform: &Mat4) {
        if self.sprites_age.is_empty() {
            return;
        }

        let scale = 50.0f32;
        let mut model = *muzzle_transform * Mat4::from_scale(vec3(scale, scale, scale));
        model *= Mat4::from_rotation_x(-90.0f32.to_radians());
        model *= Mat4::from_translation(vec3(0.7f32, 0.0f32, 0.0f32)); // adjust for position in the texture

        update_mat4_buffer(context, &self.transform_buffer, &model);

        for i in 0..self.sprites_age.len() {
            self.sprites_age[i] += delta_time;
        }
        let max_age = self.impact_spritesheet.uniform.num_columns * self.impact_spritesheet.uniform.time_per_sprite;

        self.sprites_age.retain(|age| *age < max_age);

        update_uniform_buffer(context, &self.age_buffer, self.sprites_age.as_slice());
    }

    pub fn get_min_age(&self) -> f32 {
        let mut min_age = 1000f32;
        for age in self.sprites_age.iter() {
            min_age = min_age.min(*age);
        }
        min_age
    }

    pub fn add_flash(&mut self) {
        self.sprites_age.push(0.0);
    }



    pub fn draw(&self, projection_view: &Mat4, muzzle_transform: &Mat4) {
        if self.sprites_age.is_empty() {
            return;
        }

        /*
        for muzzle sprites, we need just one uniform with num_columns and time_per_sprite
        which are features of the sprite itself

        Then we render an array of sprite instances, each instance has an age.
        Age is a feature of the instance.

        so we need one sprite uniform with num_columns and time_per_sprite
        and one buffer for instances which is a vec of ages.

        for bullets, we need the sprite uniform and
            all_bullet_positions
            all_bullet_rotations

        Does bullet shader need age? or is that fixed?

         */

        // sprite_shader.use_shader();
        // sprite_shader.set_mat4("PV", projection_view);

        // unsafe {
        //     gl::Enable(gl::BLEND);
        //     gl::DepthMask(gl::FALSE);
        //     gl::BindVertexArray(self.unit_square_vao as GLuint);
        // }
        //
        // bind_texture(sprite_shader, 0, "spritesheet", &self.muzzle_flash_impact_spritesheet.texture);

        // sprite_shader.set_int("numCols", self.muzzle_flash_impact_spritesheet.num_columns);
        // sprite_shader.set_float("timePerSprite", self.muzzle_flash_impact_spritesheet.time_per_sprite);

        let scale = 50.0f32;
        let mut model = *muzzle_transform * Mat4::from_scale(vec3(scale, scale, scale));
        model *= Mat4::from_rotation_x(-90.0f32.to_radians());
        model *= Mat4::from_translation(vec3(0.7f32, 0.0f32, 0.0f32)); // adjust for position in the texture

        // sprite_shader.set_mat4("model", &model);

        for sprite_age in self.sprites_age.iter() {
            // sprite_shader.set_float("age", *sprite_age);
            // unsafe {
            //     gl::DrawArrays(gl::TRIANGLES, 0, 6);
            // }
        }

        // unsafe {
        //     gl::Disable(gl::BLEND);
        //     gl::DepthMask(gl::TRUE);
        // }
    }


}



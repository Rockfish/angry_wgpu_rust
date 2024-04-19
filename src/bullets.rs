use std::f32::consts::PI;
use std::mem;

use glam::{vec3, vec4, Mat4, Quat, Vec3, Vec4Swizzles};
use spark_gap::gpu_context::GpuContext;
use spark_gap::material::Material;
use spark_gap::texture_config::{TextureConfig, TextureFilter, TextureType, TextureWrap};
use wgpu::util::DeviceExt;
use wgpu::Buffer;

use crate::aabb::Aabb;
use crate::capsule::Capsule;
use crate::enemy::{Enemy, ENEMY_COLLIDER};
use crate::geom::{distance_between_line_segments, oriented_angle};
use crate::render::buffers::{create_vertex_buffer, create_vertex_buffer_init, update_uniform_buffer};
use crate::small_mesh::SmallMesh;
use crate::sprite_sheet::{SpriteSheet, SpriteSheetSprite};
use crate::world::{World, MAX_BULLET_GROUPS, SPREAD_AMOUNT};

pub struct BulletGroup {
    start_index: usize,
    group_size: i32,
    time_to_live: f32,
}

impl BulletGroup {
    pub const fn new(start_index: usize, group_size: i32, time_to_live: f32) -> Self {
        Self {
            start_index,
            group_size,
            time_to_live,
        }
    }
}

pub struct BulletSystem {
    pub bullet_positions: Vec<Vec3>,
    pub bullet_rotations: Vec<Quat>,
    pub bullet_directions: Vec<Vec3>,
    pub bullet_groups: Vec<BulletGroup>,

    // fixed size calculation vecs
    x_rotations: Vec<Quat>,
    y_rotations: Vec<Quat>,

    pub impact_mesh: SmallMesh,
    pub bullet_material: Material,

    pub impact_spritesheet: SpriteSheet,
    pub impact_sprites: Vec<SpriteSheetSprite>,

    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,

    pub bullet_positions_buffer: Buffer,
    pub bullet_rotations_buffer: Buffer,
}

// const BULLET_SCALE: f32 = 0.3;
const BULLET_SCALE: f32 = 0.3;
const BULLET_LIFETIME: f32 = 1.0;
// seconds
// const BULLET_SPEED: f32 = 15.0;
const BULLET_SPEED: f32 = 5.0;
// const BULLET_SPEED: f32 = 1.0;
// Game units per second
const ROTATION_PER_BULLET: f32 = 3.0 * PI / 180.0;

const SCALE_VEC: Vec3 = vec3(BULLET_SCALE, BULLET_SCALE, BULLET_SCALE);
const BULLET_NORMAL: Vec3 = vec3(0.0, 1.0, 0.0);
const CANONICAL_DIR: Vec3 = vec3(0.0, 0.0, 1.0);

const BULLET_COLLIDER: Capsule = Capsule { height: 0.3, radius: 0.03 };

const BULLET_ENEMY_MAX_COLLISION_DIST: f32 = BULLET_COLLIDER.height / 2.0 + BULLET_COLLIDER.radius + ENEMY_COLLIDER.height / 2.0 + ENEMY_COLLIDER.radius;

const MAX_BULLETS: usize = (SPREAD_AMOUNT * SPREAD_AMOUNT * MAX_BULLET_GROUPS) as usize;

// Trim off margin around the bullet image
// const TEXTURE_MARGIN: f32 = 0.0625;
// const TEXTURE_MARGIN: f32 = 0.2;
const TEXTURE_MARGIN: f32 = 0.1;

#[rustfmt::skip]
const BULLET_VERTICES_H: [f32; 20] = [
    // Positions                                        // Tex Coords
    BULLET_SCALE * (-0.243), 0.0, BULLET_SCALE * (-1.0),  1.0 - TEXTURE_MARGIN, 0.0 + TEXTURE_MARGIN,
    BULLET_SCALE * (-0.243), 0.0, BULLET_SCALE * 0.0,     0.0 + TEXTURE_MARGIN, 0.0 + TEXTURE_MARGIN,
    BULLET_SCALE * 0.243,    0.0, BULLET_SCALE * 0.0,     0.0 + TEXTURE_MARGIN, 1.0 - TEXTURE_MARGIN,
    BULLET_SCALE * 0.243,    0.0, BULLET_SCALE * (-1.0),  1.0 - TEXTURE_MARGIN, 1.0 - TEXTURE_MARGIN,
];

// vertical surface to see the bullets from the side
#[rustfmt::skip]
const BULLET_VERTICES_V: [f32; 20] = [
    0.0, BULLET_SCALE * (-0.243), BULLET_SCALE * (-1.0),  1.0 - TEXTURE_MARGIN, 0.0 + TEXTURE_MARGIN,
    0.0, BULLET_SCALE * (-0.243), BULLET_SCALE * 0.0,     0.0 + TEXTURE_MARGIN, 0.0 + TEXTURE_MARGIN,
    0.0, BULLET_SCALE * 0.243,    BULLET_SCALE * 0.0,     0.0 + TEXTURE_MARGIN, 1.0 - TEXTURE_MARGIN,
    0.0, BULLET_SCALE * 0.243,    BULLET_SCALE * (-1.0),  1.0 - TEXTURE_MARGIN, 1.0 - TEXTURE_MARGIN,
];

#[rustfmt::skip]
const BULLET_VERTICES_H_V: [f32; 40] = [
    // Positions                                          // Tex Coords
    BULLET_SCALE * (-0.243), 0.0, BULLET_SCALE * (-1.0),  1.0 - TEXTURE_MARGIN, 0.0 + TEXTURE_MARGIN,
    BULLET_SCALE * (-0.243), 0.0, BULLET_SCALE * 0.0,     0.0 + TEXTURE_MARGIN, 0.0 + TEXTURE_MARGIN,
    BULLET_SCALE * 0.243,    0.0, BULLET_SCALE * 0.0,     0.0 + TEXTURE_MARGIN, 1.0 - TEXTURE_MARGIN,
    BULLET_SCALE * 0.243,    0.0, BULLET_SCALE * (-1.0),  1.0 - TEXTURE_MARGIN, 1.0 - TEXTURE_MARGIN,
    0.0, BULLET_SCALE * (-0.243), BULLET_SCALE * (-1.0),  1.0 - TEXTURE_MARGIN, 0.0 + TEXTURE_MARGIN,
    0.0, BULLET_SCALE * (-0.243), BULLET_SCALE * 0.0,     0.0 + TEXTURE_MARGIN, 0.0 + TEXTURE_MARGIN,
    0.0, BULLET_SCALE * 0.243,    BULLET_SCALE * 0.0,     0.0 + TEXTURE_MARGIN, 1.0 - TEXTURE_MARGIN,
    0.0, BULLET_SCALE * 0.243,    BULLET_SCALE * (-1.0),  1.0 - TEXTURE_MARGIN, 1.0 - TEXTURE_MARGIN,
];

#[rustfmt::skip]
const BULLET_INDICES: [i32; 6] = [
    0, 1, 2,
    0, 2, 3
];

#[rustfmt::skip]
const BULLET_INDICES_H_V: [u32; 12] = [
    0, 1, 2,
    0, 2, 3,
    4, 5, 6,
    4, 6, 7,
];

impl BulletSystem {
    pub fn new(context: &mut GpuContext, impact_mesh: SmallMesh) -> Self {
        let texture_config = TextureConfig {
            flip_v: false,
            flip_h: true,
            gamma_correction: false,
            filter: TextureFilter::Nearest,
            texture_type: TextureType::None,
            wrap: TextureWrap::Repeat,
        };

        let bullet_material = Material::new(context, "angrygl_assets/bullet/bullet_texture_transparent.png", &texture_config).unwrap();

        let vertices = BULLET_VERTICES_H_V;
        let indices = BULLET_INDICES_H_V;

        let vertex_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bullet mesh vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bullet mesh index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let impact_sprite_sheet_material = Material::new(context, "angrygl_assets/bullet/impact_spritesheet_with_00.png", &texture_config).unwrap();
        let impact_spritesheet = SpriteSheet::new(context, impact_sprite_sheet_material, 11.0, 0.05);

        let bullet_positions_buffer = create_vertex_buffer(context, mem::size_of::<Vec3>() * MAX_BULLETS, "bullet positions buffer");
        let bullet_rotations_buffer = create_vertex_buffer(context, mem::size_of::<Quat>() * MAX_BULLETS, "bullet rotations buffer");

        // Pre-calculate the bullet spread rotations. Only needs to be done once.
        let mut x_rotations = Vec::with_capacity(SPREAD_AMOUNT as usize);
        let mut y_rotations = Vec::with_capacity(SPREAD_AMOUNT as usize);
        let spread_centering = ROTATION_PER_BULLET * (SPREAD_AMOUNT as f32 - 1.0) / 4.0;

        for i in 0..SPREAD_AMOUNT {
            let y_rot = Quat::from_axis_angle(
                vec3(0.0, 1.0, 0.0),
                ROTATION_PER_BULLET.mul_add((i - SPREAD_AMOUNT) as f32 / 2.0, spread_centering),
            );
            let x_rot = Quat::from_axis_angle(
                vec3(1.0, 0.0, 0.0),
                ROTATION_PER_BULLET.mul_add((i - SPREAD_AMOUNT) as f32 / 2.0, spread_centering),
            );
            x_rotations.push(x_rot);
            y_rotations.push(y_rot)
        }

        Self {
            bullet_positions: vec![],
            bullet_rotations: vec![],
            bullet_directions: Default::default(),
            bullet_groups: vec![],
            x_rotations,
            y_rotations,
            bullet_material,
            impact_spritesheet,
            impact_sprites: vec![],
            impact_mesh,
            vertex_buffer,
            index_buffer,
            bullet_positions_buffer,
            bullet_rotations_buffer,
        }
    }

    pub fn create_bullets(&mut self, dx: f32, dz: f32, muzzle_transform: &Mat4, spread_amount: i32) -> bool {
        // limit number of bullet groups
        if self.bullet_groups.len() >= MAX_BULLET_GROUPS as usize {
            return false;
        }
        // let spreadAmount = 100;

        let muzzle_world_position = *muzzle_transform * vec4(0.0, 0.0, 0.0, 1.0);

        let projectile_spawn_point = muzzle_world_position.xyz();

        let mid_direction = vec3(dx, 0.0, dz).normalize();

        let normalized_direction = mid_direction.normalize_or_zero();

        let rot_vec = vec3(0.0, 1.0, 0.0); // rotate around y

        let x = vec3(CANONICAL_DIR.x, 0.0, CANONICAL_DIR.z).normalize_or_zero();
        let y = vec3(normalized_direction.x, 0.0, normalized_direction.z).normalize_or_zero();

        // direction angle with respect to the canonical direction
        let theta = oriented_angle(x, y, rot_vec) * -1.0;

        let mut mid_dir_quat = Quat::from_xyzw(1.0, 0.0, 0.0, 0.0);
        mid_dir_quat *= Quat::from_axis_angle(rot_vec, theta.to_radians());

        let start_index = self.bullet_positions.len();

        let bullet_group_size = spread_amount * spread_amount;

        let bullet_group = BulletGroup::new(start_index, bullet_group_size, BULLET_LIFETIME);

        self.bullet_positions.resize(start_index + bullet_group_size as usize, Vec3::default());
        self.bullet_rotations.resize(start_index + bullet_group_size as usize, Quat::default());
        self.bullet_directions.resize(start_index + bullet_group_size as usize, Vec3::default());

        // let i_start = 0;
        // let i_end = spread_amount;
        //
        // let spread_centering = ROTATION_PER_BULLET * (spread_amount as f32 - 1.0) / 4.0;

        // for i in i_start..i_end {
        //     let y_quat = mid_dir_quat
        //         * Quat::from_axis_angle(
        //             vec3(0.0, 1.0, 0.0),
        //             ROTATION_PER_BULLET.mul_add((i - spread_amount) as f32 / 2.0, spread_centering),
        //         );
        //
        //     for j in 0..spread_amount {
        //         let rot_quat = y_quat
        //             * Quat::from_axis_angle(
        //                 vec3(1.0, 0.0, 0.0),
        //                 ROTATION_PER_BULLET.mul_add((j - spread_amount) as f32 / 2.0, spread_centering),
        //             );
        //
        //         let direction = rot_quat.mul_vec3(CANONICAL_DIR * -1.0);
        //
        //         let index = (i * spread_amount + j) as usize + start_index;
        //
        //         self.bullet_positions[index] = projectile_spawn_point;
        //         self.bullet_directions[index] = direction;
        //         self.bullet_rotations[index] = rot_quat;
        //     }
        // }

        let start: usize = start_index;
        let end = start + bullet_group_size as usize;

        for index in start..end {
            let count = index - start;
            let i = (count as f32 / SPREAD_AMOUNT as f32).floor();
            let j = count as f32 % SPREAD_AMOUNT as f32;

            let y_quat = mid_dir_quat * self.y_rotations[i as usize];
            let rot_quat = y_quat * self.x_rotations[j as usize];
            let direction = rot_quat.mul_vec3(CANONICAL_DIR * -1.0);

            self.bullet_positions[index] = projectile_spawn_point;
            self.bullet_rotations[index] = rot_quat;
            self.bullet_directions[index] = direction;
        }

        self.bullet_groups.push(bullet_group);

        true
    }

    pub fn update_bullets(&mut self, context: &GpuContext, world: &mut World) {
        let use_aabb = !world.enemies.is_empty();
        let num_sub_groups = if use_aabb { 9 } else { 1 };

        let delta_position_magnitude = world.delta_time * BULLET_SPEED;

        let mut first_live_bullet_group: usize = 0;

        for group in self.bullet_groups.iter_mut() {
            group.time_to_live -= world.delta_time;

            if group.time_to_live <= 0.0 {
                first_live_bullet_group += 1;
            } else {
                // could make this async
                let bullet_group_start_index = group.start_index as i32;
                let num_bullets_in_group = group.group_size;
                let sub_group_size = num_bullets_in_group / num_sub_groups;

                for sub_group in 0..num_sub_groups {
                    let mut bullet_start = sub_group_size * sub_group;

                    let mut bullet_end = if sub_group == (num_sub_groups - 1) {
                        num_bullets_in_group
                    } else {
                        bullet_start + sub_group_size
                    };

                    bullet_start += bullet_group_start_index;
                    bullet_end += bullet_group_start_index;

                    for bullet_index in bullet_start..bullet_end {
                        self.bullet_positions[bullet_index as usize] += self.bullet_directions[bullet_index as usize] * delta_position_magnitude;
                    }

                    let mut subgroup_bound_box = Aabb::new();

                    if use_aabb {
                        for bullet_index in bullet_start..bullet_end {
                            subgroup_bound_box.expand_to_include(self.bullet_positions[bullet_index as usize]);
                        }

                        subgroup_bound_box.expand_by(BULLET_ENEMY_MAX_COLLISION_DIST);
                    }

                    for i in 0..world.enemies.len() {
                        let enemy = &mut world.enemies[i];

                        if use_aabb && !subgroup_bound_box.contains_point(enemy.position) {
                            continue;
                        }
                        for bullet_index in bullet_start..bullet_end {
                            if bullet_collides_with_enemy(
                                &self.bullet_positions[bullet_index as usize],
                                &self.bullet_directions[bullet_index as usize],
                                enemy,
                            ) {
                                // println!("killed enemy!");
                                enemy.is_alive = false;
                                break;
                            }
                        }
                    }
                }
            }
        }

        let mut first_live_bullet: usize = 0;

        if first_live_bullet_group != 0 {
            first_live_bullet =
                self.bullet_groups[first_live_bullet_group - 1].start_index + self.bullet_groups[first_live_bullet_group - 1].group_size as usize;
            self.bullet_groups.drain(0..first_live_bullet_group);
        }

        if first_live_bullet != 0 {
            self.bullet_positions.drain(0..first_live_bullet);
            self.bullet_directions.drain(0..first_live_bullet);
            self.bullet_rotations.drain(0..first_live_bullet);

            for group in self.bullet_groups.iter_mut() {
                group.start_index -= first_live_bullet;
            }
        }

        if !self.impact_sprites.is_empty() {
            for sheet in self.impact_sprites.iter_mut() {
                sheet.age += &world.delta_time;
            }
            let sprite_duration = self.impact_spritesheet.uniform.num_columns as f32 * self.impact_spritesheet.uniform.time_per_sprite;

            self.impact_sprites.retain(|sprite| sprite.age < sprite_duration);
        }

        for enemy in world.enemies.iter() {
            if !enemy.is_alive {
                self.impact_sprites.push(SpriteSheetSprite::new(enemy.position));
                world.burn_marks.add_mark(enemy.position);
                // world.sound_system.play_enemy_destroyed();
            }
        }

        update_uniform_buffer(context, &self.bullet_positions_buffer, &self.bullet_positions.as_slice());
        update_uniform_buffer(context, &self.bullet_rotations_buffer, &self.bullet_rotations.as_slice());
    }

    pub fn draw_bullets(&mut self, projection_view: &Mat4) {
        if self.bullet_positions.is_empty() {
            return;
        }

        // unsafe {
        //     gl::Enable(gl::BLEND);
        //     gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        //     gl::DepthMask(gl::FALSE);
        //     gl::Disable(gl::CULL_FACE);
        // }

        // shader.use_shader();
        // shader.set_mat4("PV", projection_view);
        // shader.set_bool("useLight", false);

        // bind_texture(shader, 0, "texture_diffuse", &self.bullet_texture);
        // bind_texture(shader, 1, "texture_normal", &self.bullet_texture);

        self.render_bullet_sprites();

        // unsafe {
        //     gl::Disable(gl::BLEND);
        //     gl::Enable(gl::CULL_FACE);
        //     gl::DepthMask(gl::TRUE);
        // }
    }

    pub fn render_bullet_sprites(&self) {
        /*
        unsafe {
            gl::BindVertexArray(self.bullet_vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.rotation_vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.all_bullet_rotations.len() * SIZE_OF_QUAT) as GLsizeiptr,
                self.all_bullet_rotations.as_ptr() as *const GLvoid,
                gl::STREAM_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, self.offset_vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.all_bullet_positions.len() * SIZE_OF_VEC3) as GLsizeiptr,
                self.all_bullet_positions.as_ptr() as *const GLvoid,
                gl::STREAM_DRAW,
            );

            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                12, // 6,
                gl::UNSIGNED_INT,
                NULL,
                self.all_bullet_positions.len() as GLsizei,
            );
        }
         */
    }

    // needs layout, buffers, and render
    pub fn draw_bullet_impacts(&self, world: &World, projection_view: &Mat4) {
        // sprite_shader.set_mat4("PV", projection_view);
        // sprite_shader.set_int("numCols", self.bullet_impact_spritesheet.num_columns);
        // sprite_shader.set_float("timePerSprite", self.bullet_impact_spritesheet.time_per_sprite);

        // bind_texture(sprite_shader, 0, "spritesheet", &self.bullet_impact_spritesheet.texture);

        // unsafe {
        //     gl::Enable(gl::BLEND);
        // gl::DepthMask(gl::FALSE);
        // gl::Disable(gl::CULL_FACE);
        //
        // gl::BindVertexArray(self.unit_square_vao as GLuint);
        // }

        let scale = 2.0f32; // 0.25f32;

        for sprite in &self.impact_sprites {
            let mut model = Mat4::from_translation(sprite.world_position);
            model *= Mat4::from_rotation_x(-90.0f32.to_radians());

            // TODO: Billboarding
            // for (int i = 0; i < 3; i++)
            // {
            //     for (int j = 0; j < 3; j++)
            //     {
            //         model[i][j] = viewTransform[j][i];
            //     }
            // }

            model *= Mat4::from_scale(vec3(scale, scale, scale));

            // sprite_shader.set_float("age", sprite.age);
            // sprite_shader.set_mat4("model", &model);

            // unsafe {
            //     gl::DrawArrays(gl::TRIANGLES, 0, 6);
            // }
        }

        // unsafe {
        //     gl::Disable(gl::BLEND);
        //     gl::Enable(gl::CULL_FACE);
        //     gl::DepthMask(gl::TRUE);
        // }
    }
}

fn bullet_collides_with_enemy(position: &Vec3, direction: &Vec3, enemy: &Enemy) -> bool {
    if position.distance(enemy.position) > BULLET_ENEMY_MAX_COLLISION_DIST {
        return false;
    }

    let a0 = *position - *direction * (BULLET_COLLIDER.height / 2.0);
    let a1 = *position + *direction * (BULLET_COLLIDER.height / 2.0);
    let b0 = enemy.position - enemy.direction * (ENEMY_COLLIDER.height / 2.0);
    let b1 = enemy.position + enemy.direction * (ENEMY_COLLIDER.height / 2.0);

    let closet_distance = distance_between_line_segments(&a0, &a1, &b0, &b1);

    closet_distance <= (BULLET_COLLIDER.radius + ENEMY_COLLIDER.radius)
}

pub fn rotate_by_quat(v: &Vec3, q: &Quat) -> Vec3 {
    let q_prime = Quat::from_xyzw(q.w, -q.x, -q.y, -q.z);
    partial_hamilton_product(&partial_hamilton_product2(q, v), &q_prime)
}

pub fn partial_hamilton_product2(quat: &Quat, vec: &Vec3 /*partial*/) -> Quat {
    Quat::from_xyzw(
        quat.w * vec.x + quat.y * vec.z - quat.z * vec.y,
        quat.w * vec.y - quat.x * vec.z + quat.z * vec.x,
        quat.w * vec.z + quat.x * vec.y - quat.y * vec.x,
        -quat.x * vec.x - quat.y * vec.y - quat.z * vec.z,
    )
}

pub fn partial_hamilton_product(q1: &Quat, q2: &Quat) -> Vec3 {
    vec3(
        q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
        q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
        q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
    )
}

fn hamilton_product_quat_vec(quat: &Quat, vec: &Vec3) -> Quat {
    Quat::from_xyzw(
        quat.w * vec.x + quat.y * vec.z - quat.z * vec.y,
        quat.w * vec.y - quat.x * vec.z + quat.z * vec.x,
        quat.w * vec.z + quat.x * vec.y - quat.y * vec.x,
        -quat.x * vec.x - quat.y * vec.y - quat.z * vec.z,
    )
}

fn hamilton_product_quat_quat(first: Quat, other: &Quat) -> Quat {
    Quat::from_xyzw(
        first.w * other.x + first.x * other.w + first.y * other.z - first.z * other.y,
        first.w * other.y - first.x * other.z + first.y * other.w + first.z * other.x,
        first.w * other.z + first.x * other.y - first.y * other.x + first.z * other.w,
        first.w * other.w - first.x * other.x - first.y * other.y - first.z * other.z,
    )
}

#[cfg(test)]
mod tests {
    use glam::vec3;

    use crate::geom::oriented_angle;

    #[test]
    fn test_oriented_rotation() {
        let canonical_dir = vec3(0.0, 0.0, -1.0);

        for angle in 0..361 {
            let (sin, cos) = (angle as f32).to_radians().sin_cos();
            let x = sin;
            let z = cos;

            let direction = vec3(x, 0.0, z);

            let normalized_direction = direction; //.normalize_or_zero();

            let rot_vec = vec3(0.0, 1.0, 0.0); // rotate around y

            let x = vec3(canonical_dir.x, 0.0, canonical_dir.z).normalize_or_zero();
            let y = vec3(normalized_direction.x, 0.0, normalized_direction.z).normalize_or_zero();

            // direction angle with respect to the canonical direction
            let theta = oriented_angle(x, y, rot_vec) * -1.0;

            println!("angle: {}  direction: {:?}   theta: {:?}", angle, normalized_direction, theta);
        }
    }
}

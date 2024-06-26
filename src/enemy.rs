use std::char::MAX;
use std::f32::consts::PI;
use std::mem;

use glam::{vec2, vec3, Mat4, Vec3};
use spark_gap::gpu_context::GpuContext;
use spark_gap::model::Model;
use spark_gap::model_builder::ModelBuilder;
use spark_gap::utils::rand_float;
use wgpu::{BindGroup, Buffer, BufferAddress};

use crate::capsule::Capsule;
use crate::geom::distance_between_point_and_line_segment;
use crate::render::buffers::{
    create_buffer_bind_group, create_uniform_bind_group_layout, create_uniform_buffer, create_uniform_buffer_init, create_vertex_buffer_init,
    get_or_create_bind_group_layout, update_uniform_buffer,
};
use crate::small_mesh::SmallMeshVertex;
use crate::world::{World, MONSTER_SPEED, MONSTER_Y, PLAYER_COLLISION_RADIUS};

pub const MAX_ENEMIES: usize = 100;
pub const ENEMY_COLLIDER: Capsule = Capsule { height: 0.4, radius: 0.08 };
const ENEMY_SPAWN_INTERVAL: f32 = 1.0; // seconds
const SPAWNS_PER_INTERVAL: i32 = 1;
const SPAWN_RADIUS: f32 = 10.0; // from player

pub const ENEMY_UNIFORMS_BIND_GROUP_LAYOUT: &str = "enemy instances bind group layout";

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EnemyUniform {
    model_transform: Mat4,
    aim_rotation: Mat4,
}

pub struct Enemy {
    pub position: Vec3,
    pub direction: Vec3,
    pub is_alive: bool,
}

pub struct EnemySystem {
    pub count_down: f32,
    pub monster_y: f32,
    pub enemy_model: Model,
    pub instances_uniforms: Vec<EnemyUniform>,
    pub instances_uniform_buffer: Buffer,
    pub instances_bind_group: BindGroup,
}

impl EnemySystem {
    pub fn new(context: &mut GpuContext) -> Self {
        // EelDog model has diffuse and height materials
        let enemy_model = ModelBuilder::new("enemy", "assets/Models/Eeldog/EelDog.FBX").build(context).unwrap();

        let instances_uniform_buffer = create_uniform_buffer(context, mem::size_of::<EnemyUniform>() * MAX_ENEMIES, "enemies instances uniform vec");

        let layout = get_or_create_bind_group_layout(context, ENEMY_UNIFORMS_BIND_GROUP_LAYOUT, create_uniform_bind_group_layout);
        let instances_bind_group = create_buffer_bind_group(context, &layout, &instances_uniform_buffer, "enemies instances bind group");

        Self {
            count_down: ENEMY_SPAWN_INTERVAL,
            monster_y: MONSTER_Y,
            enemy_model,
            instances_uniforms: vec![],
            instances_uniform_buffer,
            instances_bind_group,
        }
    }

    pub fn update(&mut self, context: &mut GpuContext, world: &mut World) {
        world.enemies.retain(|e| e.is_alive);

        self.count_down -= world.delta_time;

        if self.count_down <= 0.0 {
            for _i in 0..SPAWNS_PER_INTERVAL {
                self.spawn_enemy(world)
            }
            self.count_down += ENEMY_SPAWN_INTERVAL;
        }

        self.instances_uniforms.clear();

        for e in world.enemies.iter_mut() {
            let monster_theta = (e.direction.x / e.direction.z).atan() + (if e.direction.z < 0.0 { 0.0 } else { PI });

            let mut model_transform = Mat4::from_translation(e.position);

            model_transform *= Mat4::from_scale(Vec3::splat(0.01));
            model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monster_theta);
            model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
            model_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

            let aim_rotation = Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

            let uniform = EnemyUniform { model_transform, aim_rotation };

            self.instances_uniforms.push(uniform);
        }

        update_uniform_buffer(context, &self.instances_uniform_buffer, self.instances_uniforms.as_slice());
    }

    pub fn spawn_enemy(&mut self, world: &mut World) {
        if world.enemies.len() == MAX_ENEMIES {
            return;
        }

        let theta = (rand_float() * 360.0).to_radians();
        let x = theta.sin().mul_add(SPAWN_RADIUS, world.player.borrow().position.x);
        let z = theta.cos().mul_add(SPAWN_RADIUS, world.player.borrow().position.z);

        let position = vec3(x, self.monster_y, z);
        let mut dir = world.player.borrow_mut().position - position;
        dir.y = 0.0;

        let enemy = Enemy {
            position,
            direction: dir.normalize_or_zero(),
            is_alive: true,
        };

        world.enemies.push(enemy);
    }

    pub fn chase_player(&self, world: &mut World) {
        let mut player = world.player.borrow_mut();
        let player_collision_position = vec3(player.position.x, MONSTER_Y, player.position.z);

        for enemy in world.enemies.iter_mut() {
            let mut dir = player.position - enemy.position;
            dir.y = 0.0;
            enemy.direction = dir.normalize_or_zero();
            enemy.position += enemy.direction * world.delta_time * MONSTER_SPEED;

            if player.is_alive {
                let p1 = enemy.position - enemy.direction * (ENEMY_COLLIDER.height / 2.0);
                let p2 = enemy.position + enemy.direction * (ENEMY_COLLIDER.height / 2.0);
                let dist = distance_between_point_and_line_segment(&player_collision_position, &p1, &p2);

                if dist <= (PLAYER_COLLISION_RADIUS + ENEMY_COLLIDER.radius) {
                    // println!("GOTTEM!");
                    player.is_alive = false;
                    player.set_player_death_time(world.frame_time);
                    player.direction = vec2(0.0, 0.0);
                }
            }
        }
    }
}

// fn create_enemy_instances_bind_group_layout(context: &GpuContext) -> BindGroupLayout {
//     context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//         entries: &[
//             wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStages::VERTEX,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new((MAX_ENEMIES * 16 * 2) as _),
//                 },
//                 count: None,
//             },
//         ],
//         label: Some("enemy instances bind group layout"),
//     })
// }

// fn create_enemy_instances_bind_group(
//     context: &GpuContext,
//     bind_group_layout: &BindGroupLayout,
//     enemy_instances: &Buffer,
// ) -> BindGroup {
//     context.device.create_bind_group(&wgpu::BindGroupDescriptor {
//         layout: bind_group_layout,
//         entries: &[
//             wgpu::BindGroupEntry {
//                 binding: 2,
//                 resource: enemy_instances.as_entire_binding(),
//             },
//         ],
//         label: Some("enemy instances bind group"),
//     })
// }

use crate::geom::distance_between_point_and_line_segment;
use glam::{vec2, vec3, Mat4, Vec3};
use std::f32::consts::PI;
use spark_gap::gpu_context::GpuContext;
use spark_gap::model::Model;
use spark_gap::model_builder::ModelBuilder;
use spark_gap::utils::rand_float;
use crate::capsule::Capsule;
use crate::world::{MONSTER_SPEED, MONSTER_Y, PLAYER_COLLISION_RADIUS, World};

pub const ENEMY_COLLIDER: Capsule = Capsule { height: 0.4, radius: 0.08 };

pub struct Enemy {
    pub position: Vec3,
    pub dir: Vec3,
    pub is_alive: bool,
}

impl Enemy {
    pub const fn new(position: Vec3, dir: Vec3) -> Self {
        Self { position, dir, is_alive: true }
    }
}

const ENEMY_SPAWN_INTERVAL: f32 = 1.0; // seconds
const SPAWNS_PER_INTERVAL: i32 = 1;
const SPAWN_RADIUS: f32 = 10.0; // from player

pub struct EnemySystem {
    pub count_down: f32,
    pub monster_y: f32,
    pub enemy_model: Model,
}

impl EnemySystem {
    pub fn new(context: &mut GpuContext) -> Self {
        let enemy_model = ModelBuilder::new("enemy", "assets/Models/Eeldog/EelDog.FBX").build(context).unwrap();
        Self {
            count_down: ENEMY_SPAWN_INTERVAL,
            monster_y: MONSTER_Y,
            enemy_model,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        self.count_down -= world.delta_time;
        if self.count_down <= 0.0 {
            for _i in 0..SPAWNS_PER_INTERVAL {
                self.spawn_enemy(world)
            }
            self.count_down += ENEMY_SPAWN_INTERVAL;
        }
    }

    pub fn spawn_enemy(&mut self, world: &mut World) {
        let theta = (rand_float() * 360.0).to_radians();
        // let x = state.player.borrow().position.x + theta.sin() * SPAWN_RADIUS;
        // let z = state.player.borrow().position.z + theta.cos() * SPAWN_RADIUS;
        let x = theta.sin().mul_add(SPAWN_RADIUS, world.player.borrow().position.x);
        let z = theta.cos().mul_add(SPAWN_RADIUS, world.player.borrow().position.z);
        world.enemies.push(Enemy::new(vec3(x, self.monster_y, z), vec3(0.0, 0.0, 1.0)));
    }

    pub fn chase_player(&self, world: &mut World) {
        let mut player = world.player.borrow_mut();
        let player_collision_position = vec3(player.position.x, MONSTER_Y, player.position.z);

        for enemy in world.enemies.iter_mut() {
            let mut dir = player.position - enemy.position;
            dir.y = 0.0;
            enemy.dir = dir.normalize_or_zero();
            enemy.position += enemy.dir * world.delta_time * MONSTER_SPEED;

            if player.is_alive {
                let p1 = enemy.position - enemy.dir * (ENEMY_COLLIDER.height / 2.0);
                let p2 = enemy.position + enemy.dir * (ENEMY_COLLIDER.height / 2.0);
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

    pub fn draw_enemies(&self, world: &mut World) {
        // shader.use_shader();
        // shader.set_vec3("nosePos", &vec3(1.0, MONSTER_Y, -2.0));
        // shader.set_float("time", world.frame_time);

        // TODO optimise (multithreaded, instancing, SOA, etc..)
        for e in world.enemies.iter_mut() {
            let monster_theta = (e.dir.x / e.dir.z).atan() + (if e.dir.z < 0.0 { 0.0 } else { PI });

            let mut model_transform = Mat4::from_translation(e.position);

            model_transform *= Mat4::from_scale(Vec3::splat(0.01));
            model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monster_theta);
            model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
            model_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

            // let mut rot_only = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monster_theta);
            // rot_only = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
            let rot_only = Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

            // shader.set_mat4("aimRot", &rot_only);
            // shader.set_mat4("model", &model_transform);

            // self.enemy_model.render(shader);
        }
    }
}



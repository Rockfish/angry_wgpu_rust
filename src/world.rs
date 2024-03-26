use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use glam::{Mat4, Vec3};
use spark_gap::camera::camera::Camera;
use spark_gap::camera::camera_handler::CameraHandler;
use spark_gap::camera::fly_camera_controller::FlyCameraController;
use spark_gap::hash_map::HashSet;
use spark_gap::input::Input;
use winit::keyboard::Key;

use crate::bullets::BulletSystem;
use crate::burn_marks::BurnMarks;
use crate::enemy::{Enemy, EnemySystem};
use crate::floor::Floor;
use crate::game_loop::CameraType;
// use crate::params::floor_lighting::FloorLightingHandler;
use crate::muzzle_flash::MuzzleFlash;
use crate::params::shader_params::ShaderParametersHandler;
use crate::player::Player;
use crate::render::main_render::WorldRender;

pub const FIRE_INTERVAL: f32 = 0.1;
// seconds
pub const SPREAD_AMOUNT: i32 = 20;
pub const MAX_BULLET_GROUPS: i32 = 10;

pub const PLAYER_COLLISION_RADIUS: f32 = 0.35;

// Models
pub const PLAYER_MODEL_SCALE: f32 = 0.0044;
//const PLAYER_MODEL_GUN_HEIGHT: f32 = 120.0; // un-scaled
pub const PLAYER_MODEL_GUN_HEIGHT: f32 = 110.0;
// un-scaled
pub const PLAYER_MODEL_GUN_MUZZLE_OFFSET: f32 = 100.0;
// un-scaled
pub const MONSTER_Y: f32 = PLAYER_MODEL_SCALE * PLAYER_MODEL_GUN_HEIGHT;

// Lighting
pub const LIGHT_FACTOR: f32 = 0.8;
pub const NON_BLUE: f32 = 0.9;

pub const BLUR_SCALE: i32 = 2;

pub const FLOOR_LIGHT_FACTOR: f32 = 0.35;
pub const FLOOR_NON_BLUE: f32 = 0.7;

// Enemies
pub const MONSTER_SPEED: f32 = 0.6;

pub struct World {
    pub camera_controller: FlyCameraController,
    pub camera_handler: CameraHandler,
    pub camera_follow_vec: Vec3,
    pub run: bool,
    pub window_scale: (f32, f32),
    pub key_presses: HashSet<Key>,
    pub game_camera: Camera,
    pub floating_camera: Camera,
    pub ortho_camera: Camera,
    pub active_camera: CameraType,
    pub game_projection: Mat4,
    pub floating_projection: Mat4,
    pub orthographic_projection: Mat4,
    pub start_instant: Instant,
    pub delta_time: f32,
    pub frame_time: f32,
    pub first_mouse: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub input: Input,
    pub player: RefCell<Player>,
    pub scene_render: RefCell<WorldRender>,
    pub shader_params: ShaderParametersHandler,
    pub floor: RefCell<Floor>,
    pub enemy_system: Rc<RefCell<EnemySystem>>,
    pub muzzle_flash: Rc<RefCell<MuzzleFlash>>,
    pub bullet_system: Rc<RefCell<BulletSystem>>,
    pub enemies: Vec<Enemy>,
    pub burn_marks: BurnMarks,
    // pub sound_system: SoundSystem,
    pub light_direction: Vec3,
}

impl World {
    pub fn update_time(&mut self) {
        let current_time = Instant::now().duration_since(self.start_instant).as_secs_f32();
        if self.run {
            self.delta_time = current_time - self.frame_time;
        } else {
            self.delta_time = 0.0;
        }
        self.frame_time = current_time;
    }

    pub fn handle_input(&mut self) {
        if let Some(mouse_position) = self.input.mouse_position {
            self.mouse_x = mouse_position.x;
            self.mouse_y = mouse_position.y;
        }
    }
}

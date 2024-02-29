use std::cell::RefCell;
use std::rc::Rc;
use glam::{Mat4, Vec3};
use spark_gap::camera::camera_handler::CameraHandler;
use spark_gap::camera::fly_camera_controller::FlyCameraController;
use spark_gap::input::Input;
use std::time::Instant;
use spark_gap::camera::camera::Camera;
use spark_gap::hash_map::HashSet;
use wgpu::TextureView;
use winit::keyboard::Key;
use crate::game_loop::CameraType;
use crate::lighting::GameLightingHandler;
use crate::player::Player;
use crate::render::anim_render::AnimRenderPass;
use crate::sound_system::SoundSystem;

pub struct World {
    pub camera_controller: FlyCameraController,
    pub camera_handler: CameraHandler,
    pub camera_follow_vec: Vec3,
    pub player: RefCell<Player>,
    pub player_render: RefCell<AnimRenderPass>,
    pub model_transform: Mat4,
    pub game_lighting_handler: GameLightingHandler,
    // pub depth_texture_view: TextureView,
    pub run: bool,
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub scaled_width: i32,
    pub scaled_height: i32,
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
    // pub player: Rc<RefCell<Player>>,
    // pub enemies: Vec<Enemy>,
    // pub burn_marks: BurnMarks,
    pub sound_system: SoundSystem,
    pub buffer_ready: bool,
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
}
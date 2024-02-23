use glam::{Mat4, Vec3};
use spark_gap::camera::camera_handler::CameraHandler;
use spark_gap::camera::fly_camera_controller::FlyCameraController;
use spark_gap::input::Input;
use spark_gap::model::Model;
use std::time::Instant;
use wgpu::TextureView;

pub struct World {
    pub camera_controller: FlyCameraController,
    pub camera_handler: CameraHandler,
    pub model: Model,
    pub model_2: Model,
    pub model_position: Vec3,
    pub model_transform: Mat4,
    pub depth_texture_view: TextureView,
    pub run: bool,
    pub start_instant: Instant,
    pub delta_time: f32,
    pub frame_time: f32,
    pub first_mouse: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub input: Input,
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

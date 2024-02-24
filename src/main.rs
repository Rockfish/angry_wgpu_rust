mod game_loop;
mod world;
mod sound_system;
mod geom;
mod sprite_sheet;
mod aabb;
mod bullets;
mod bullets_parallel;
mod burn_marks;
mod enemy;
mod floor;
mod framebuffers;
mod game;
mod muzzle_flash;
mod player;
mod quads;
mod render;
mod lighting;

use std::sync::Arc;
use winit::event_loop::EventLoop;
use crate::game_loop::run;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    #[allow(unused_mut)]
        let mut builder = winit::window::WindowBuilder::new();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        builder = builder.with_canvas(Some(canvas));
    }

    let window = Arc::new(
        builder
            .with_title("Animation Example")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0))
            .build(&event_loop)
            .unwrap(),
    );

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run(event_loop, window));
    }

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
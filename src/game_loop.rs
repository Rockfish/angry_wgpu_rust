use std::cell::RefCell;
use std::rc::Rc;
use crate::render::anim_render::{create_depth_texture_view, AnimRenderPass};
use crate::world::{State, World};
use glam::{vec3, Mat4, Vec3};
use spark_gap::camera::camera_handler::CameraHandler;
use spark_gap::camera::fly_camera_controller::FlyCameraController;
use spark_gap::frame_counter::FrameCounter;
use spark_gap::gpu_context::GpuContext;
use spark_gap::input::Input;
use spark_gap::model_builder::ModelBuilder;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard;
use winit::keyboard::NamedKey::Escape;
use winit::window::Window;
use crate::lighting::{GameLightingHandler, GameLightingUniform};
use crate::player::Player;

pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

pub async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let mut context = GpuContext::new(window).await;
    let mut frame_counter = FrameCounter::new();
    let size = context.window.inner_size();
    let aspect_ratio = size.width as f32 / size.height as f32;

    let camera_position = vec3(0.0, 100.0, 300.0);
    let camera_controller = FlyCameraController::new(aspect_ratio, camera_position, 0.0, 0.0);
    let camera_handler = CameraHandler::new(&mut context, &camera_controller);

    let game_lighting_uniform = GameLightingUniform::new();
    let game_lighting_handler = GameLightingHandler::new(&mut context, game_lighting_uniform);

    // let model_path = "examples/animation/vampire/dancing_vampire.dae";
    // let model = ModelBuilder::new("model", model_path).build(&mut context).unwrap();
    // let model_2 = ModelBuilder::new("model", model_path).build(&mut context).unwrap();

    let model = Player::new(&mut context);

    let model_position = Vec3::ZERO;

    let depth_texture_view = create_depth_texture_view(&context);

    let anim_render = AnimRenderPass::new(&mut context);

    #[allow(unused_mut)]
    let mut model_transform = Mat4::IDENTITY;
    model_transform *= Mat4::from_translation(vec3(0.0, 0.0, 1.0));
    // model *= Mat4::from_rotation_x(-90.0f32.to_radians());
    // model_transform *= Mat4::from_translation(vec3(0.0, -10.4, -200.0));
    // model *= Mat4::from_scale(vec3(0.3, 0.3, 0.3));
    // let mut model = Mat4::from_translation(vec3(0.0, 5.0, 0.0));
    // model_transform *= Mat4::from_scale(vec3(15.0, 15.0, 15.0));
    // model_transform *= Mat4::from_scale(vec3(1.0, 1.0, 1.0));

    let mut world = World {
        camera_controller,
        camera_handler,
        model,
        model_position,
        model_transform,
        game_lighting_handler,
        depth_texture_view,
        run: true,
        // start_instant: Instant::now(),
        // delta_time: 0.0,
        // frame_time: 0.0,
        first_mouse: false,
        mouse_x: 0.0,
        mouse_y: 0.0,
        input: Input::default(),
    };
    
    let mut state = State {
        run: false,
        viewport_width: 0,
        viewport_height: 0,
        scaled_width: 0,
        scaled_height: 0,
        window_scale: (0.0, 0.0),
        key_presses: Default::default(),
        // game_camera: FlyCameraController {},
        // floating_camera: FlyCameraController {},
        // ortho_camera: FlyCameraController {},
        game_projection: Default::default(),
        floating_projection: Default::default(),
        orthographic_projection: Default::default(),
        start_instant: Instant::now(),
        delta_time: 0.0,
        frame_time: 0.0,
        first_mouse: false,
        mouse_x: 0.0,
        mouse_y: 0.0,
        // player: Rc::new(RefCell::new(model)),
        // sound_system: (),
    };

    event_loop
        .run(move |event, target| {
            match event {
                Event::WindowEvent { event, .. } => {
                    world.input.handle_window_event(&event);
                    match event {
                        WindowEvent::Resized(new_size) => {
                            context.resize(new_size);
                            world.camera_controller.resize(&context);
                            world.camera_handler.update_camera(&context, &world.camera_controller);
                            world.depth_texture_view = create_depth_texture_view(&context);
                            context.window.request_redraw();
                        }
                        WindowEvent::RedrawRequested => {
                            frame_counter.update();

                            state.update_time();

                            world.camera_controller.update(&world.input, state.delta_time);
                            world.camera_handler.update_camera(&context, &world.camera_controller);

                            // world.model.update_animation(world.delta_time - 0.004);
                            world.model.update(&state, 1.0);

                            anim_render.render(&context, &world);

                            context.window.request_redraw();

                            // println!("Input: {:#?}\n", &world.input);
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            // if event.state == ElementState::Pressed {
                            if event.logical_key == keyboard::Key::Named(Escape) {
                                target.exit()
                            } else {
                            }
                            // }
                        }
                        WindowEvent::CloseRequested => target.exit(),
                        _ => {}
                    }
                }
                Event::DeviceEvent { event, .. } => {
                    world.input.handle_device_event(&event);
                }
                _ => {}
            }
        })
        .unwrap();
}

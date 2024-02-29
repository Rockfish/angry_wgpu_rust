use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;
use crate::render::main_render::{create_depth_texture_view, AnimRenderPass};
use crate::world::{FLOOR_LIGHT_FACTOR, FLOOR_NON_BLUE, LIGHT_FACTOR, NON_BLUE, PLAYER_MODEL_SCALE, World};
use glam::{vec3, Mat4, Vec3, vec4};
use spark_gap::camera::camera_handler::CameraHandler;
use spark_gap::camera::fly_camera_controller::FlyCameraController;
use spark_gap::frame_counter::FrameCounter;
use spark_gap::gpu_context::GpuContext;
use spark_gap::input::Input;
use spark_gap::model_builder::ModelBuilder;
use std::sync::Arc;
use std::time::Instant;
use spark_gap::camera::camera::Camera;
use spark_gap::math::{get_world_ray_from_mouse, ray_plane_intersection};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard;
use winit::keyboard::NamedKey::Escape;
use winit::window::Window;
use crate::burn_marks::BurnMarks;
use crate::enemy::EnemySystem;
use crate::lighting::{DirectionLight, GameLightingHandler, GameLightingUniform, PointLight};
use crate::player::Player;
use crate::sound_system::SoundSystem;

const PARALLELISM: i32 = 4;

// Viewport
const VIEW_PORT_WIDTH: i32 = 1500;
const VIEW_PORT_HEIGHT: i32 = 1000;
// const VIEW_PORT_WIDTH: i32 = 800;
// const VIEW_PORT_HEIGHT: i32 = 500;

// Player

pub enum CameraType {
    Game,
    Floating,
    TopDown,
    Side,
}

pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

pub async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    info!("Game started.");
    info!("Loading assets");

    let mut context = GpuContext::new(window).await;
    let mut frame_counter = FrameCounter::new();

    let size = context.window.inner_size();

    let aspect_ratio = size.width as f32 / size.height as f32;

    let mut viewport_width = size.width as f32;
    let mut viewport_height = size.height as f32;
    let mut scaled_width = (viewport_width / 1.0) as i32;
    let mut scaled_height = (viewport_height / 1.0) as i32;

    info!(
        "initial view port size: {}, {}  scaled size: {}, {}",
        viewport_width, viewport_height, scaled_width, scaled_height
    );

    // --- Lighting ---

    let light_dir: Vec3 = vec3(-0.8, 0.0, -1.0).normalize_or_zero();
    let player_light_dir: Vec3 = vec3(-1.0, -1.0, -1.0).normalize_or_zero();
    let muzzle_point_light_color = vec3(1.0, 0.2, 0.0);

    let light_color: Vec3 = LIGHT_FACTOR * 1.0 * vec3(NON_BLUE * 0.406, NON_BLUE * 0.723, 1.0);
    let ambient_color: Vec3 = LIGHT_FACTOR * 0.10 * vec3(NON_BLUE * 0.7, NON_BLUE * 0.7, 0.7);

    let floor_light_color: Vec3 = FLOOR_LIGHT_FACTOR * 1.0 * vec3(FLOOR_NON_BLUE * 0.406, FLOOR_NON_BLUE * 0.723, 1.0);
    let floor_ambient_color: Vec3 = FLOOR_LIGHT_FACTOR * 0.50 * vec3(FLOOR_NON_BLUE * 0.7, FLOOR_NON_BLUE * 0.7, 0.7);

    // --- Cameras ---

    let camera_follow_vec = vec3(-4.0, 4.3, 0.0);
    let _camera_up = vec3(0.0, 1.0, 0.0);

    let game_camera = Camera::camera_vec3_up_yaw_pitch(
        vec3(0.0, 20.0, 80.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0,
        -20.0,
    );

    let floating_camera = Camera::camera_vec3_up_yaw_pitch(
        vec3(0.0, 10.0, 20.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0,
        -20.0,
    );

    let ortho_camera = Camera::camera_vec3_up_yaw_pitch(vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0), 0.0, -90.0);

    let ortho_width = VIEW_PORT_WIDTH as f32 / 130.0;
    let ortho_height = VIEW_PORT_HEIGHT as f32 / 130.0;
    let aspect_ratio = VIEW_PORT_WIDTH as f32 / VIEW_PORT_HEIGHT as f32;
    let game_projection = Mat4::perspective_rh_gl(game_camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let floating_projection = Mat4::perspective_rh_gl(floating_camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let orthographic_projection = Mat4::orthographic_rh_gl(-ortho_width, ortho_width, -ortho_height, ortho_height, 0.1, 100.0);


    let camera_position = vec3(0.0, 100.0, 300.0);
    let camera_controller = FlyCameraController::new(aspect_ratio, camera_position, 0.0, 0.0);
    let camera_handler = CameraHandler::new(&mut context, &camera_controller);

    let direction_light = DirectionLight {
        direction: player_light_dir,
        color: light_color,
    };

    let point_light = PointLight {
        world_pos: Default::default(),
        color: Default::default(),
    };

    let game_lighting_uniform = GameLightingUniform{
        direction_light,
        point_light,
        aim_rotation: Mat4::IDENTITY,
        light_space_matrix: Mat4::IDENTITY,
        view_position: vec3(100.0, 100.0, 300.0),
        ambient_color,
        depth_mode: 0,
        use_point_light: 1,
        use_light: 1,
        use_emissive: 1,
        _pad: [0.0; 6],
    };

    let game_lighting_handler = GameLightingHandler::new(&mut context, game_lighting_uniform);


    let mut player = Player::new(&mut context);
    // let floor = Floor::new();
    let mut enemies = EnemySystem::new(&mut context);
    // let mut muzzle_flash = MuzzleFlash::new(unit_square_quad);
    // let mut bullet_store = BulletStore::new(unit_square_quad);



    let player_render = AnimRenderPass::new(&mut context);

    let model_transform = Mat4::from_translation(vec3(0.0, 0.0, 1.0));

    let mut world = World {
        camera_controller,
        camera_handler,
        camera_follow_vec,
        player: player.into(),
        player_render: player_render.into(),
        model_transform,
        game_lighting_handler,
        // depth_texture_view,
        run: false,
        viewport_width: 0,
        viewport_height: 0,
        scaled_width: 0,
        scaled_height: 0,
        window_scale: (0.0, 0.0),
        key_presses: Default::default(),
        game_camera,
        floating_camera,
        ortho_camera,
        active_camera: CameraType::Game,
        game_projection,
        floating_projection,
        orthographic_projection,
        start_instant: Instant::now(),
        delta_time: 0.0,
        frame_time: 0.0,
        first_mouse: false,
        mouse_x: 0.0,
        mouse_y: 0.0,
        input: Input::default(),
        enemies: vec![],
        burn_marks: BurnMarks::new(&mut context, 0),
        sound_system: SoundSystem::new(),
        buffer_ready: false,
    };

    event_loop
        .run(move |event, target| {
            match event {
                Event::WindowEvent { event, .. } => {
                    world.input.handle_window_event(&event);
                    match event {
                        WindowEvent::RedrawRequested => {
                            frame_counter.update();
                            world.update_time();

                            world.camera_controller.update(&world.input, world.delta_time);
                            world.camera_handler.update_camera(&context, &world.camera_controller);

                            game_run(&context, &mut world);

                            context.window.request_redraw();

                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            // if event.state == ElementState::Pressed {
                            if event.logical_key == keyboard::Key::Named(Escape) {
                                target.exit()
                            } else {
                            }
                            // }
                        }
                        WindowEvent::Resized(new_size) => {
                            context.resize(new_size);
                            world.camera_controller.resize(&context);
                            world.camera_handler.update_camera(&context, &world.camera_controller);
                            // world.depth_texture_view = create_depth_texture_view(&context);
                            context.window.request_redraw();
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

fn game_run(context: &GpuContext, mut world: &mut World) {

    world.game_camera.position = world.player.borrow().position + world.camera_follow_vec;
    let game_view = Mat4::look_at_rh(world.game_camera.position, world.player.borrow().position, world.game_camera.up);

    let (projection, camera_view) = match world.active_camera {
        CameraType::Game => (world.game_projection, game_view),
        CameraType::Floating => {
            let view = Mat4::look_at_rh(world.floating_camera.position, world.player.borrow().position, world.floating_camera.up);
            (world.floating_projection, view)
        }
        CameraType::TopDown => {
            let view = Mat4::look_at_rh(
                vec3(world.player.borrow().position.x, 1.0, world.player.borrow().position.z),
                world.player.borrow().position,
                vec3(0.0, 0.0, -1.0),
            );
            (world.orthographic_projection, view)
        }
        CameraType::Side => {
            let view = Mat4::look_at_rh(vec3(0.0, 0.0, -3.0), world.player.borrow().position, vec3(0.0, 1.0, 0.0));
            (world.orthographic_projection, view)
        }
    };

    let projection_view = projection * camera_view;

    let mut dx: f32 = 0.0;
    let mut dz: f32 = 0.0;
    let mut aim_theta = 0.0f32;

    if world.player.borrow().is_alive && world.buffer_ready {
        let world_ray = get_world_ray_from_mouse(
            world.mouse_x,
            world.mouse_y,
            world.scaled_width as f32,
            world.scaled_height as f32,
            &game_view,
            &world.game_projection,
        );

        let xz_plane_point = vec3(0.0, 0.0, 0.0);
        let xz_plane_normal = vec3(0.0, 1.0, 0.0);

        let world_point = ray_plane_intersection(world.game_camera.position, world_ray, xz_plane_point, xz_plane_normal).unwrap();

        dx = world_point.x - world.player.borrow().position.x;
        dz = world_point.z - world.player.borrow().position.z;
        aim_theta = (dx / dz).atan() + if dz < 0.0 { PI } else { 0.0 };

        if world.mouse_x.abs() < 0.005 && world.mouse_y.abs() < 0.005 {
            aim_theta = 0.0;
        }
    }

    let aim_rot = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), aim_theta);

    let mut player_transform = Mat4::from_translation(world.player.borrow().position);
    player_transform *= Mat4::from_scale(Vec3::splat(PLAYER_MODEL_SCALE));
    player_transform *= aim_rot;

    let muzzle_transform = world.player.borrow().get_muzzle_position(&player_transform);

    /*
    if world.player.borrow().is_alive && world.player.borrow().is_trying_to_fire && (world.player.borrow().last_fire_time + FIRE_INTERVAL) < world.frame_time {
        world.player.borrow_mut().last_fire_time = world.frame_time;
        if bullet_store.create_bullets(dx, dz, &muzzle_transform, SPREAD_AMOUNT) {
            muzzle_flash.add_flash();
            world.sound_system.play_player_shooting();
        }
    }

    muzzle_flash.update(world.delta_time);
    bullet_store.update_bullets(&mut world);

    if world.player.borrow().is_alive {
        enemies.update(&mut world);
        enemies.chase_player(&mut world);
    }
     */

    // Update world.player
    world.player.borrow_mut().update(&world, aim_theta);

    let mut use_point_light = false;
    let mut muzzle_world_position = Vec3::default();

    /*
    if !muzzle_flash.muzzle_flash_sprites_age.is_empty() {
        let min_age = muzzle_flash.get_min_age();
        let muzzle_world_position_vec4 = muzzle_transform * vec4(0.0, 0.0, 0.0, 1.0);

        muzzle_world_position = vec3(
            muzzle_world_position_vec4.x / muzzle_world_position_vec4.w,
            muzzle_world_position_vec4.y / muzzle_world_position_vec4.w,
            muzzle_world_position_vec4.z / muzzle_world_position_vec4.w,
        );

        use_point_light = min_age < 0.03;
    }
     */

    let near_plane: f32 = 1.0;
    let far_plane: f32 = 50.0;
    let ortho_size: f32 = 10.0;
    let player_position = world.player.borrow().position;

    let player_light_dir = world.game_lighting_handler.uniform.direction_light.direction;

    let light_projection = Mat4::orthographic_rh_gl(-ortho_size, ortho_size, -ortho_size, ortho_size, near_plane, far_plane);
    let light_view = Mat4::look_at_rh(player_position - 20.0 * player_light_dir, player_position, vec3(0.0, 1.0, 0.0));
    let light_space_matrix = light_projection * light_view;

    world.game_lighting_handler.uniform.light_space_matrix = light_space_matrix;
    world.game_lighting_handler.uniform.use_point_light = if use_point_light { 1 } else { 0 };

    world.player.borrow().model.borrow().update_animation(world.delta_time - 0.004);
    world.player.borrow_mut().update(&world, 1.0);

    world.player_render.borrow_mut().render(&context, &world);

    // world.buffer_ready = true;
}

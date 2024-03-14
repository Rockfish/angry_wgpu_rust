#define_import_path spark::instanced_shader
#import spark::common::{CameraUniform, DirectionLight};

// shaders/instanced_texture_shader.vert
// shaders/basic_texture_shader.frag

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(3) rotationQuat: vec4<f32>,
    @location(4) positionOffset: vec3<f32>,
}

struct SpriteUniform {
    number_of_columns: f32,
    time_per_sprite: f32,
};

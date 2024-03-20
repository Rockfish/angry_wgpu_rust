#define_import_path spark::bullet_shader
#import spark::common::{CameraUniform};

// Bullet instances
// shaders/instanced_texture_shader.vert
// shaders/basic_texture_shader.frag

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct PositionInput {
    @location(2) position: vec3<f32>,
}

struct RotationInput {
    @location(3) rotation: vec4<f32>,
}

struct SpriteUniform {
    number_of_columns: f32,
    time_per_sprite: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// material information
@group(1) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(1) @binding(1) var diffuse_sampler: sampler;

fn hamiltonProduct(q1: vec4<f32>, q2: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
    q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
    q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
    q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
    q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z
    );
}

fn multiplyQuaternions(q1: vec4<f32>, q2: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
    q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y, // w
    q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x, // x
    q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w, // y
    q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z  // z
    );
}

// glm stores quat as { w, x, y, z }
// glam stores quat as { x, y, z, w }
fn flip(glam: vec4<f32>) -> vec4<f32> {
    var glm = vec4<f32>(glam.w, glam.x, glam.y, glam.z);
    return glm;
}

fn rotate_by_quat(v: vec3<f32>, q_orig: vec4<f32>) -> vec3<f32> {

    var q = flip(q_orig); // flip convention;

    var qPrime = vec4<f32>(-q.x, -q.y, -q.z, q.w);

    var first = hamiltonProduct(q, vec4<f32>(v.x, v.y, v.z, 0.0));
    var vPrime = hamiltonProduct(first, qPrime);

    return vec3<f32>(vPrime.x, vPrime.y, vPrime.z);
}

@vertex fn vs_main(
    vert_in: VertexInput,
    position_in: PositionInput,
    rotation_in: RotationInput
) -> VertexOutput {

    var result: VertexOutput;

    var rotated_in_position = rotate_by_quat(vert_in.position, rotation_in.rotation);

    result.position = camera.projection * camera.view * vec4<f32>(rotated_in_position + position_in.position, 1.0);
    result.tex_coords = vert_in.tex_coords;

    return result;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    return color;
}
#define_import_path spark::bullet_shader
#import spark::common::{CameraUniform};

// Bullet instances
// shaders/instanced_texture_shader.vert
// shaders/basic_texture_shader.frag

const SPREAD_AMOUNT: u32 = 20;
const MAX_BULLET_GROUPS: u32 = 10;
const MAX_BULLETS: u32 = SPREAD_AMOUNT * SPREAD_AMOUNT * MAX_BULLET_GROUPS;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

// instance index into postion and rotation arrays
struct InstanceInput {
    @location(7) index: u32,
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

@group(1) @binding(0) var<uniform> bullet_positions: array<vec3<f32>, MAX_BULLETS>;
@group(2) @binding(0) var<uniform> bullet_rotations: array<vec4<f32>, MAX_BULLETS>;

// material information
@group(3) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(3) @binding(1) var diffuse_sampler: sampler;

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

@vertex fn vs_main(in: VertexInput, instance: InstanceInput) -> VertexOutput {

    var result: VertexOutput;
    var rotation_quat = bullet_rotations[instance.index];
    var position_offset = bullet_positions[instance.index];

    var rotated_in_position = rotate_by_quat(in.position, rotation_quat);

    result.position = camera.projection * camera.view * vec4<f32>(rotated_in_position + position_offset, 1.0);
    result.tex_coords = in.tex_coords;

    return result;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    return color;
}
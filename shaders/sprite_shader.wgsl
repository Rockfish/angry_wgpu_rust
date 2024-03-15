#define_import_path spark::sprite_shader
#import spark::common::{CameraUniform};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

// age vec
struct InstanceInput {
    @location(3) age: f32,
}

struct SpriteUniform {
    number_of_columns: f32,
    time_per_sprite: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;

@group(2) @binding(0) var sprite_texture: texture_2d<f32>;
@group(2) @binding(1) var sprite_sampler: sampler;

@group(3) @binding(0) var<uniform> sprite: SpriteUniform;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) age: f32,
};

@vertex fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {

    var result: VertexOutput;

    var in_position = vec4<f32>(input.position, 1.0);
    result.position = camera.projection * camera.view * model_transform * in_position;
    result.tex_coords = input.tex_coords;
    result.age = instance.age;

    return result;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  var col = floor(in.age / sprite.time_per_sprite);
  var sprite_tex_coords = vec2<f32>(in.tex_coords.x / sprite.number_of_columns + col * (1 / sprite.number_of_columns), in.tex_coords.y);
  var color = textureSample(sprite_texture, sprite_sampler, sprite_tex_coords);
  return color;
}
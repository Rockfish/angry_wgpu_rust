#define_import_path spark::debug_depth_shader
#import spark::common::{CameraUniform};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// camera
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// model
@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;

// shadow texture
//@group(2) @binding(0) var texture: texture_depth_2d;
@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;


@vertex fn vs_main(vertex_input: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    result.position = camera.projection * camera.view * model_transform * vec4<f32>(vertex_input.position, 1.0);
    result.tex_coords = vertex_input.tex_coords;

    return result;
}


@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//    let texCoord = coord.xy * 0.5 + 0.5;  // Transform from [-1, 1] to [0, 1] range
    let value = textureSample(texture, texture_sampler, in.tex_coords);
//    let depth = result.z;
//    return vec4<f32>(value, value, value, 1.0);  // Visualize depth as grayscale
    return value;
}


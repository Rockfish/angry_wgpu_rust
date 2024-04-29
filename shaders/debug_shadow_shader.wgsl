#define_import_path spark::debug_depth_shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0) var<uniform> projection_view: mat4x4<f32>;
@group(0) @binding(1) var<uniform> model_transform: mat4x4<f32>;
@group(0) @binding(2) var<uniform> layer_num: u32;

@group(0) @binding(3) var texture: texture_depth_2d_array;
@group(0) @binding(4) var texture_sampler: sampler;


@vertex fn vs_main(vertex_input: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    result.position = projection_view * model_transform * vec4<f32>(vertex_input.position, 1.0);
    result.tex_coords = vertex_input.tex_coords;

    return result;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let flip_correction = vec2<f32>(1.0, -1.0);
    let tex_coords = in.tex_coords * flip_correction + vec2<f32>(0.0, 1.0);

    var value = textureSample(texture, texture_sampler, tex_coords, layer_num);

    // expand top range and reverse the range for better grayscale contrast
    value = 1.0 - (value - 0.80) * 5.0;

    return vec4<f32>(value, value, value, 1.0);
}

#define_import_path spark::animation_shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent: vec2<f32>,
    @location(4) bitangent: vec2<f32>,
    @location(5) bone_ids: vec4<i32>,
    @location(6) weights: vec4<f32>,
}

struct CameraUniform {
   projection: mat4x4<f32>,
   view: mat4x4<f32>,
   position: vec3<f32>,
}

const MAX_BONES = 100;
const MAX_BONE_INFLUENCE = 4;

// camera
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// model transforms
@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;
@group(1) @binding(1) var<uniform> node_transform: mat4x4<f32>;
@group(1) @binding(2) var<uniform> bone_transforms: array<mat4x4<f32>, MAX_BONES>;

// material information
@group(2) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1) var diffuse_sampler: sampler;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// Vertex shader
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    var anim_position = get_animated_position(model);

    result.position = camera.projection * camera.view * model_transform * anim_position;
    result.tex_coords = model.tex_coords;

    return result;
}

fn get_animated_position(model: VertexInput) -> vec4<f32> {
    var initial_postion = vec4<f32>(0.0);
    var totalPosition = initial_postion;
    var localNormal = vec3<f32>(0.0);

    for (var i = 0; i < MAX_BONE_INFLUENCE; i++)
    {
        if (model.bone_ids[i] == -1) {
            continue;
        }

        if (model.bone_ids[i] >= MAX_BONES) {
            totalPosition = vec4<f32>(model.position, 1.0f);
            break;
        }

        var localPosition = bone_transforms[model.bone_ids[i]] * vec4<f32>(model.position, 1.0f);
        totalPosition += localPosition * model.weights[i];

//    localNormal = mat3(bone_transforms[bone_ids[i]]) * norm;
    }

    if (all(totalPosition == initial_postion)) {
        totalPosition = node_transform * vec4<f32>(model.position, 1.0f);
    }

    return totalPosition;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    return color;
}

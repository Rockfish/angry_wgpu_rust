
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) bone_ids: vec4<i32>,
    @location(6) weights: vec4<f32>,
}

struct CameraUniform {
   projection: mat4x4<f32>,
   view: mat4x4<f32>,
   position: vec3<f32>,
}

struct AnimationOutput {
    position: vec4<f32>,
    local_normal: vec3<f32>,
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

// game and lighting
@group(3) @binding(0) var<uniform> aim_rotation: mat4x4<f32>;
@group(3) @binding(1) var<uniform> depth_mode: bool;
@group(3) @binding(2) var<uniform> light_space_matrix: mat4x4<f32>;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) frag_world_position: vec2<f32>,
    @location(2) frag_light_space_position: vec2<f32>,
};

// Vertex shader
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    var anim_position = get_animated_position(model);

    result.position = camera.projection * camera.view * model_transform * anim_position;
    result.tex_coords = model.tex_coords;

    result.frag_world_position = vec3(model_transform * vec4(model.position, 1.0));
    result.light_space_position = light_space_matrix * vec4(result.frag_world_position, 1.0);

    return result;
}

fn get_animated_position(model: VertexInput) -> AnimationOutput {

    var output: AnimationOutput;

    var initial_postion = vec4<f32>(0.0);

    output.position = initial_postion;
    output.local_normal = vec3<f32>(0.0);

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

        output.position += localPosition * model.weights[i];
        output.local_normal = mat3(bone_transforms[bone_ids[i]]) * model.normal;
    }

    if (all(output.position == initial_postion)) {
        output.position = node_transform * vec4<f32>(model.position, 1.0f);
    }

    return output;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    return color;
}

fn ShadowCalculation(bias: f32, fragPosLightSpace: vec4<f32>) -> f32 {

  var projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
  projCoords = projCoords * 0.5 + 0.5;

  var closestDepth = texture(shadow_map, projCoords.xy).r;
  var currentDepth = projCoords.z;

  bias = 0.001;
  var shadow = 0.0;

  if (currentDepth - bias) > closestDepth {
    shadown = 1.0;
  };

  return shadow;
}
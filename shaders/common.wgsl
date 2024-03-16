#define_import_path spark::common

const PLAYER_MODEL_SCALE: f32 = 0.0044;
const PLAYER_MODEL_GUN_HEIGHT: f32 = 110.0;
const PLAYER_MODEL_GUN_MUZZLE_OFFSET: f32 = 100.0;
const MONSTER_Y: f32 = PLAYER_MODEL_SCALE * PLAYER_MODEL_GUN_HEIGHT;

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

struct DirectionLight {
    direction: vec4<f32>,
    color: vec4<f32>,
}

struct PointLight {
    world_position: vec4<f32>,
    color: vec4<f32>,
}

const MAX_BONES = 100;
const MAX_BONE_INFLUENCE = 4;

struct ModelTransforms {
    model_transform: mat4x4<f32>,
    node_transform: mat4x4<f32>,
    bone_transforms: array<mat4x4<f32>, 100>,
}

struct AnimationOutput {
    position: vec4<f32>,
    local_normal: vec3<f32>,
}

struct ShaderParameters {
    direction_light: DirectionLight,
    point_light: PointLight,
    model_rotation: mat4x4<f32>,
    light_space_matrix: mat4x4<f32>,
    view_position: vec4<f32>,
    ambient_color: vec4<f32>,
    time: f32,
    depth_mode: i32,
    use_light: i32,
    use_point_light: i32,
    use_emissive: i32,
    use_specular: i32,
}

fn get_animated_position(
    model: VertexInput,
    node_transform: mat4x4<f32>,
    bone_transforms: array<mat4x4<f32>, 100>
) -> AnimationOutput {

    var output: AnimationOutput;

    // this assignment is a workaround for a naga error-
    // "expression may only be indexed by a constant"
    var bones = bone_transforms;

    var initial_postion = vec4<f32>(0.0);

    output.position = initial_postion;
    output.local_normal = vec3<f32>(0.0);

    for (var i = 0; i < MAX_BONE_INFLUENCE; i++)
    {
        if (model.bone_ids[i] == -1) {
            continue;
        }

        if (model.bone_ids[i] >= MAX_BONES) {
            output.position = vec4<f32>(model.position, 1.0f);
            break;
        }

        var localPosition = bones[model.bone_ids[i]] * vec4<f32>(model.position, 1.0f);

        output.position += localPosition * model.weights[i];

        // todo: revisit, doesn't seem right
//        output.local_normal = mat3x3<f32>(bone_transforms[model.bone_ids[i]]) * model.normal;
    }

    if (all(output.position == initial_postion)) {
        output.position = node_transform * vec4<f32>(model.position, 1.0f);
    }

    return output;
}

//fn shadow_calculation(bias: f32, fragPosLightSpace: vec4<f32>) -> f32 {
//
//  var projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
//  projCoords = projCoords * 0.5 + 0.5;
//
//  var closestDepth = textureSample(shadow_map_texture, shadow_map_sampler, projCoords.xy).r;
//  var currentDepth = projCoords.z;
//
//  var shadow = 0.0;
//  if (currentDepth - bias) > closestDepth {
//    shadow = 1.0;
//  };
//
//  return shadow;
//}
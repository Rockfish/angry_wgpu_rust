#define_import_path spark::common

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
    direction: vec3<f32>,
    color: vec3<f32>,
}

struct PointLight {
    world_position: vec3<f32>,
    color: vec3<f32>,
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
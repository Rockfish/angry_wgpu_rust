#define_import_path spark::floor_shader
#import spark::common::{CameraUniform, DirectionLight, PointLight, ShaderParameters};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

// camera
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// model
@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;

// game and lighting
@group(2) @binding(0) var<uniform> params: ShaderParameters;

// material information
@group(3) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(3) @binding(1) var diffuse_sampler: sampler;

@group(4) @binding(0) var specular_texture: texture_2d<f32>;
@group(4) @binding(1) var specular_sampler: sampler;

@group(5) @binding(0) var normal_texture: texture_2d<f32>;
@group(5) @binding(1) var normal_sampler: sampler;

@group(6) @binding(0) var shadow_map_texture: texture_depth_2d;
@group(6) @binding(1) var shadow_map_sampler: sampler_comparison;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) fragment_world_position: vec3<f32>,
    @location(2) fragement_light_space_position: vec4<f32>,
};

@vertex fn vs_shadow(vertex_input: VertexInput) -> @builtin(position) vec4<f32> {
    return params.light_space_matrix * model_transform * vec4<f32>(vertex_input.position, 1.0);
}

// from basic_texture_shader.vert
@vertex fn vs_main(vertex_input: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    var in_position = vec4<f32>(vertex_input.position, 1.0);

    result.position = camera.projection * camera.view * model_transform * in_position;
    result.tex_coords = vertex_input.tex_coords;

    result.fragment_world_position = (model_transform * in_position).xyz;
    result.fragement_light_space_position = params.light_space_matrix * vec4<f32>(result.fragment_world_position, 1.0);

    return result;
}


@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var use_light = params.use_light;
    var use_point_light = params.use_point_light;
    var use_emissive = params.use_emissive;
    var use_specular = params.use_specular;

    var diffuse_color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    var color = diffuse_color;

    if (use_light == 1) {

        let dimensions = textureDimensions(shadow_map_texture, 0).xy;
        let texelSize = vec2<f32>(1.0, 1.0) / vec2<f32>(f32(dimensions.x), f32(dimensions.y));

        var lightDir = normalize(-params.direction_light.direction.xyz);
        var normal = textureSample(normal_texture, normal_sampler, in.tex_coords).xyz;

        normal = normalize(normal * 2.0 - 1.0);
        var diff = max(dot(normal, lightDir), 0.0);
        var amb = params.ambient_color.xyz * diffuse_color.xyz;

        var bias = max(0.05 * (1.0 - dot(normal, lightDir)), 0.005);
        var shadow = 0.0;

        for (var x = -1; x <= 1; x += 1) {
          for (var y = -1; y <= 1; y += 1) {
                let offset = vec2<f32>(f32(x), f32(y)) * texelSize;
                shadow += shadow_calculation(bias, in.fragement_light_space_position, offset);
          }
        }

        shadow /= 7.0;
        shadow *= 0.7;
        color = 0.7 * (1.0 - shadow) * params.direction_light.color * diffuse_color * diff + vec4<f32>(amb, 1.0);

        if (use_specular == 1) {
          var normal = vec3<f32>(0.0, 1.0, 0.0);
          var specLightDir = normalize(vec3<f32>(-3.0, 0.0, -1.0));
          var reflectDir = reflect(specLightDir, normal);
          var viewDir = normalize(params.view_position.xyz - in.fragment_world_position);
          var shininess = 0.7;
          var str = 1.0;//0.88;
          var spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
          color += str * spec * textureSample(specular_texture, specular_sampler, in.tex_coords) * params.direction_light.color;
        }

        if (use_point_light == 1) {
          var lightDir = normalize(params.point_light.world_position.xyz - in.fragment_world_position);
          var normal = vec3<f32>(0.0, 1.0, 0.0);
          var diff = max(dot(normal, lightDir), 0.0);
          var distance = length(params.point_light.world_position.xyz - in.fragment_world_position);
          var linear_val = 0.5;
          var constant = 0.0;
          var quadratic = 3.0;
          var attenuation = 1.0 / (constant + linear_val * distance + quadratic * (distance * distance));
          var diffuse  = params.point_light.color.xyz  * diff * diffuse_color.xyz;
          diffuse *= attenuation;
          // needs to have the opposite effect for good flash shadows
          // color += vec4(diffuse.xyz, 1.0) * (1.0 - shadow * diff); // doesn't work
          color += vec4<f32>(diffuse.xyz, 1.0);
        }
    }

    return color;
}

//fn ShadowCalculation(bias: f32, fragPosLightSpace: vec4<f32>) -> f32 {
//
//  var projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
//  projCoords = projCoords * 0.5 + 0.5;
//
//  let shadowDepth = textureSampleCompare(shadow_map_texture, shadow_map_sampler, projCoords.xy, projCoords.z);
//  var currentDepth = projCoords.z;
//
//  var shadow = 0.0;
//  if (currentDepth - bias) > shadowDepth {
//    shadow = 1.0;
//  };
//
//  return shadow;
//}

fn shadow_calculation(bias: f32, frag_light_space: vec4<f32>, offset: vec2<f32>) -> f32 {

//  let proj_correction = frag_light_space.xyz / frag_light_space.w;
//  let flip_correction = vec2<f32>(0.5, -0.5);
//
//  let projCoords = proj_correction.xy * flip_correction + vec2<f32>(0.5, 0.5);

  var projCoords = frag_light_space.xyz / frag_light_space.w;
  projCoords = projCoords * 0.5 + 0.5;

  let shadow_depth = textureSampleCompare(shadow_map_texture, shadow_map_sampler, projCoords.xy + offset, projCoords.z);

  return shadow_depth + bias;
}
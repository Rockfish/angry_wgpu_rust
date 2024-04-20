#define_import_path spark::player_shader
#import spark::common::{VertexInput, CameraUniform, DirectionLight, PointLight, ShaderParameters};
#import spark::common::{MAX_BONES, MAX_BONE_INFLUENCE, get_animated_position, AnimationOutput};


// camera
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// model and animation transforms
@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;
@group(1) @binding(1) var<uniform> node_transform: mat4x4<f32>;
@group(1) @binding(2) var<uniform> bone_transforms: array<mat4x4<f32>, MAX_BONES>;

// game and lighting
@group(2) @binding(0) var<uniform> params: ShaderParameters;

// material information
@group(3) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(3) @binding(1) var diffuse_sampler: sampler;

@group(4) @binding(0) var specular_texture: texture_2d<f32>;
@group(4) @binding(1) var specular_sampler: sampler;

@group(5) @binding(0) var emissive_texture: texture_2d<f32>;
@group(5) @binding(1) var emissive_sampler: sampler;

@group(6) @binding(0) var shadow_map_texture: texture_depth_2d;
@group(6) @binding(1) var shadow_map_sampler: sampler_comparison;


// Vertex shader section

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) light_space_position: vec4<f32>,
};

@vertex fn vs_shadow(vertex_input: VertexInput) -> @builtin(position) vec4<f32> {
    var anim_output = get_animated_position(vertex_input, node_transform, bone_transforms);
    return params.light_space_matrix * model_transform * anim_output.position;
}

@vertex fn vs_main(model: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    var anim_output = get_animated_position(model, node_transform, bone_transforms);

    result.position = camera.projection * camera.view * model_transform * anim_output.position;
    result.tex_coords = model.tex_coords;

    result.normal = (params.model_rotation * vec4<f32>(model.normal, 1.0)).xyz;

    result.world_position = (model_transform * vec4<f32>(model.position, 1.0)).xyz;
    result.light_space_position = params.light_space_matrix * vec4<f32>(result.world_position, 1.0);

    return result;
}

// Fragment shader section

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var use_light = params.use_light;
    var use_point_light = params.use_point_light;
    var use_emissive = params.use_emissive;
    var depth_mode = params.depth_mode;
    var time = params.time;

    var color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);

      if (use_light != 0) {

        var normal = normalize(in.normal);

        var shadow: f32 = 0.0;

        { // direction light

          var lightDir = normalize(-params.direction_light.direction.xyz);

          // TODO use normal texture as well
          var diff: f32 = max(dot(normal, lightDir), 0.0);
          var amb = params.ambient_color.xyz * textureSample(diffuse_texture, diffuse_sampler, in.tex_coords).xyz;
          var bias: f32 = max(0.05 * (1.0 - dot(normal, lightDir)), 0.005);

          bias = 0.001;
          shadow = ShadowCalculation(bias, in.light_space_position);

          color = (1.0 - shadow) * params.direction_light.color * color * diff + vec4<f32>(amb, 1.0);
        }

        if (use_point_light != 0) {
          var lightDir = normalize(params.point_light.world_position.xyz - in.world_position);
          var diff = max(dot(normal, lightDir), 0.0);
          var diffuse  = 0.7 * params.point_light.color.xyz * diff * textureSample(diffuse_texture, diffuse_sampler, in.tex_coords).xyz;
          color += vec4<f32>(diffuse.xyz, 1.0);
        }

        if (shadow < 0.1) {  // Spec
          var reflectDir = reflect(-params.direction_light.direction.xyz, normal);
          var viewDir = normalize(params.view_position.xyz - in.world_position);
          var shininess = 24.0;
          var str = 1.0;//0.88;

          var spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);

          color += str * spec * textureSample(specular_texture, specular_sampler, in.tex_coords) * params.direction_light.color;
          color += spec * 0.1 * vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }

        if (use_emissive != 0) {
          var emission = textureSample(emissive_texture, emissive_sampler, in.tex_coords);//.rgb;
          color += emission;
        }
      }

    return color;
}

fn ShadowCalculation(bias: f32, fragPosLightSpace: vec4<f32>) -> f32 {

  var projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
  projCoords = projCoords * 0.5 + 0.5;

  let shadowDepth = textureSampleCompare(shadow_map_texture, shadow_map_sampler, projCoords.xy, projCoords.z);
  var currentDepth = projCoords.z;

  var shadow = 0.0;
  if (currentDepth - bias) > shadowDepth {
    shadow = 1.0;
  };

  return shadow;
}
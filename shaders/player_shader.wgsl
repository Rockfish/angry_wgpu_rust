#define_import_path spark::player_shader
#import spark::common::{VertexInput, CameraUniform, DirectionLight, PointLight};
#import spark::common::{MAX_BONES, MAX_BONE_INFLUENCE, get_animated_position, AnimationOutput};

struct GameLighting {
    direction_light: DirectionLight,
    point_light: PointLight,
    aim_rotation: mat4x4<f32>,
    light_space_matrix: mat4x4<f32>,
    view_position: vec3<f32>,
    ambient_color: vec3<f32>,
    depth_mode: i32,
    use_point_light: i32,
    use_light: i32,
    use_emissive: i32,
}

// camera
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// model transforms
@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;
@group(1) @binding(1) var<uniform> node_transform: mat4x4<f32>;
@group(1) @binding(2) var<uniform> bone_transforms: array<mat4x4<f32>, MAX_BONES>;
//@group(1) @binding(0) var<uniform> model_transforms: ModelTransforms;

// game and lighting
@group(2) @binding(0) var<uniform> game_lighting: GameLighting;

// material information
@group(3) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(3) @binding(1) var diffuse_sampler: sampler;

@group(4) @binding(0) var specular_texture: texture_2d<f32>;
@group(4) @binding(1) var specular_sampler: sampler;

@group(5) @binding(0) var emissive_texture: texture_2d<f32>;
@group(5) @binding(1) var emissive_sampler: sampler;

//@group(6) @binding(0) var shadow_map_texture: texture_2d<f32>;
//@group(6) @binding(1) var shadow_map_sampler: sampler;



// Vertex shader section

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) light_space_position: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    var anim_output = get_animated_position(model, node_transform, bone_transforms);

    result.position = camera.projection * camera.view * model_transform * anim_output.position;
    result.tex_coords = model.tex_coords;

    result.normal = (game_lighting.aim_rotation * vec4<f32>(model.normal, 1.0)).xyz;

    result.world_position = (model_transform * vec4<f32>(model.position, 1.0)).xyz;
    result.light_space_position = game_lighting.light_space_matrix * vec4<f32>(result.world_position, 1.0);

    return result;
}


// Fragment shader section



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);

      if (game_lighting.use_light != 0) {

        var normal = normalize(in.normal);

        var shadow: f32 = 0.0;

        { // direction light

          var lightDir = normalize(-game_lighting.direction_light.direction);

          // TODO use normal texture as well
          var diff: f32 = max(dot(normal, lightDir), 0.0);
          var amb = game_lighting.ambient_color * textureSample(diffuse_texture, diffuse_sampler, in.tex_coords).xyz;
          var bias: f32 = max(0.05 * (1.0 - dot(normal, lightDir)), 0.005);

          bias = 0.001;
//          shadow = ShadowCalculation(bias, in.light_space_position);

          color = (1.0 - shadow) * vec4<f32>(game_lighting.direction_light.color, 1.0) * color * diff + vec4<f32>(amb, 1.0);
        }

        if (game_lighting.use_point_light != 0) {
          var lightDir = normalize(game_lighting.point_light.world_pos - in.world_position);
          var diff = max(dot(normal, lightDir), 0.0);
          var diffuse  = 0.7 * game_lighting.point_light.color  * diff * (textureSample(diffuse_texture, diffuse_sampler, in.tex_coords)).xyz;
          color += vec4<f32>(diffuse.xyz, 1.0);
        }

        if (shadow < 0.1) {  // Spec
          var reflectDir = reflect(-game_lighting.direction_light.direction, normal);
          var viewDir = normalize(game_lighting.view_position - in.world_position);
          var shininess = 24.0;
          var str = 1.0;//0.88;

          var spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);

          color += str * spec * textureSample(specular_texture, specular_sampler, in.tex_coords) * vec4<f32>(game_lighting.direction_light.color, 1.0);
          color += spec * 0.1 * vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }

        if (game_lighting.use_emissive != 0) {
          var emission = textureSample(emissive_texture, emissive_sampler, in.tex_coords);//.rgb;
          color += emission;
        }
      }

    return color;
}

//fn ShadowCalculation(bias: f32, fragPosLightSpace: vec4<f32>) -> f32 {
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
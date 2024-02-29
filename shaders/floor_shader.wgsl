#define_import_path spark::floor_shader
#import spark::common::{DirectionLight, PointLight};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;
@group(1) @binding(1) var<uniform> projection_view: mat4x4<f32>;
@group(1) @binding(2) var<uniform> light_space_matrix: mat4x4<f32>;

// material information
@group(3) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(3) @binding(1) var diffuse_sampler: sampler;

@group(4) @binding(0) var specular_texture: texture_2d<f32>;
@group(4) @binding(1) var specular_sampler: sampler;

@group(5) @binding(0) var normal_texture: texture_2d<f32>;
@group(5) @binding(1) var normal_sampler: sampler;

// lighting and shadow
@group(6) @binding(0) var<uniform> use_light: i32;
@group(6) @binding(1) var<uniform> use_spec: i32;
@group(6) @binding(2) var<uniform> ambient_color: vec3<f32>;
@group(6) @binding(3) var<uniform> view_position: vec3<f32>;
@group(6) @binding(4) var shadow_map: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(3) light_space_position: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    result.position = projection_view * model_transform * vec4<f32>(input.position, 1.0);
    result.tex_coords = input.tex_coords;
    result.light_space_position = light_space_matrix * vec4<f32>(result.position, 1.0);

    return result;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);

    if (use_light != 0) {
        var texelSize = 1.0 / textureSize(shadow_map, 0);
        var lightDir = normalize(-directionLight.dir);
        var normal = (texture(texture_normal, TexCoord)).xyz;
        normal = normalize(normal * 2.0 - 1.0);
        var diff = max(dot(normal, lightDir), 0.0);
        var amb = ambient * vec3<f32>(texture(texture_diffuse, TexCoord));
        var bias = max(0.05 * (1.0 - dot(normal, lightDir)), 0.005);
        var shadow = 0.0;

        for (var x = -1; x <= 1; x++) {
          for (var y = -1; y <= 1; y++) {
            shadow += ShadowCalculation(bias, FragPosLightSpace, vec2<f32>(x, y) * texelSize);
          }
        }

        shadow /= 7.0;
        shadow *= 0.7;
        color = 0.7 * (1.0 - shadow) * vec4<f32>(directionLight.color, 1.0) * color * diff + vec4<f32>(amb, 1.0);

        if (useSpec) {
          var normal = vec3<f32>(0.0, 1.0, 0.0);
          var specLightDir = normalize(vec3<f32>(-3.0, 0.0, -1.0));
          var reflectDir = reflect(specLightDir, normal);
          var viewDir = normalize(viewPos - FragWorldPos);
          var shininess = 0.7;
          var str = 1;//0.88;
          var spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
          color += str * spec * texture(texture_specular, TexCoord) * vec4<f32>(directionLight.color, 1.0);
        }

        if (usePointLight) {
          var lightDir = normalize(pointLight.worldPos - FragWorldPos);
          var normal = vec3<f32>(0.0, 1.0, 0.0);
          var diff = max(dot(normal, lightDir), 0.0);
          var distance = length(pointLight.worldPos- FragWorldPos);
          var linear_val = 0.5;
          var constant = 0;
          var quadratic = 3;
          var attenuation = 1.0 / (constant + linear_val * distance + quadratic * (distance * distance));
          var diffuse  = pointLight.color  * diff * vec3<f32>(texture(texture_diffuse, TexCoord));
          diffuse *= attenuation;
          // needs to have the opposite effect for good flash shadows
          // color += vec4(diffuse.xyz, 1.0) * (1.0 - shadow * diff); // doesn't work
          color += vec4<f32>(diffuse.xyz, 1.0);
        }
    }

    return color;
}

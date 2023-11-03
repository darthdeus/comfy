struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};

struct Light {
    color: vec4<f32>,
    world_position: vec2<f32>,
    screen_position: vec2<f32>,
    radius: f32,
    strength: f32,
    _padding: vec2<f32>,
}

struct LightsUniform {
    lights: array<Light, 128>,
    light_count: u32,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(1)
var<uniform> lights: LightsUniform;

@group(2) @binding(0)
var<uniform> params: GlobalLightingParams;

@group(2) @binding(1)
var color_lut_texture: texture_2d<f32>;

@group(2) @binding(2)
var color_lut_sampler: sampler;

// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.world_position = model.position;

    return out;
}


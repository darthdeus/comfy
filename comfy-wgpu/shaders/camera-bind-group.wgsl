struct GlobalLightingParams {
    ambient_light_color: vec4<f32>,
    ambient_light_intensity: f32,
    quadratic_falloff: u32,
    lighting_enabled: u32,
    shadow_strength: f32,
    fog_color: vec4<f32>,
    film_grain_strength: f32,
    noise_strength: f32,

    light_blending_mode: u32,
    global_light_intensity: f32,
    global_light_color: vec4<f32>,
    vignette_color: vec4<f32>,
    vignette_intensity: f32,
    vignette_radius: f32,
    shake_amount: f32,
    time: f32,
    color_balance: vec4<f32>,
    global_brightness: f32,
    debug_visualization: u32,
    gamma_correction: f32,
    use_lut: u32,
    exposure: f32,
    gamma: f32,
    pre_saturation: f32,
    post_saturation: f32,
    resolution: vec2<f32>,
    chromatic_aberration: f32,
    bloom_threshold: f32,
    bloom_lerp: f32,
    bloom_gamma: f32,
}

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

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(1)
var<uniform> lights: LightsUniform;

@group(1) @binding(2)
var<uniform> params: GlobalLightingParams;

@group(1) @binding(3)
var color_lut_texture: texture_2d<f32>;

@group(1) @binding(4)
var color_lut_sampler: sampler;


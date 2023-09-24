// 2D Perlin noise function
fn perlin_noise(v: vec2<f32>) -> f32 {
    var perm = array<i32, 256>(
        151,160,137,91,90,15,
        131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
        190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
        88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
        77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
        102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
        135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
        5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
        223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
        129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
        251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
        49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
        138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180
    );

    var xi = i32(floor(v.x)) & 255;
    var yi = i32(floor(v.y)) & 255;
    var xf = f32(fract(v.x));
    var yf = f32(fract(v.y));
    var u = fade(xf);
    var v = fade(yf);

    let a  = perm[(xi + 0) & 255];
    let b  = perm[(xi + 1) & 255];
    let aa = f32(perm[(a + yi + 0) & 255]);
    let ab = f32(perm[(a + yi + 1) & 255]);
    let ba = f32(perm[(b + yi + 0) & 255]);
    let bb = f32(perm[(b + yi + 1) & 255]);

    let x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1.0, yf), u);
    let x2 = lerp(grad(ab, xf, yf - 1.0), grad(bb, xf - 1.0, yf - 1.0), u);
    let y1 = lerp(x1, x2, v);

    return (y1 + 1.0) / 2.0;
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return (1.0 - t) * a + t * b;
}

fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

// fn grad(hash: f32, x: f32, y: f32) -> f32 {
//     let h = hash % 4.0;
//     let u = if (h < 2.0) { x } else { y };
//     let v = if (h < 2.0) { y } else { x };
//     return ((h % 2.0) * 2.0 - 1.0) * u + ((floor(h / 2.0) % 2.0) * 2.0 - 1.0) * v;
// }

fn grad(hash: f32, x: f32, y: f32) -> f32 {
    let h = hash % 4.0;
    let u = select(y, x, h < 2.0);
    let v = select(x, y, h < 2.0);
    return ((h % 2.0) * 2.0 - 1.0) * u + ((floor(h / 2.0) % 2.0) * 2.0 - 1.0) * v;
}

// Pseudo-random function for the film grain effect
fn pseudo_rand(v: vec2<f32>) -> f32 {
    return fract(sin(dot(v, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

// Overlay blend mode
fn blend_overlay(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return select(
        2.0 * base * blend,
        1.0 - 2.0 * (1.0 - base) * (1.0 - blend),
        base <= vec3<f32>(0.5, 0.5, 0.5)
    );
}

fn blend_multiply(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return base * blend;
}

fn blend_screen(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return 1.0 - (1.0 - base) * (1.0 - blend);
}

fn blend_soft_light(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    let d = select(
        (1.0 - 2.0 * blend) * base * (1.0 - base),
        2.0 * blend * (1.0 - (1.0 - base)),
        blend <= vec3<f32>(0.5, 0.5, 0.5)
    );
    return base + d;
}

fn blend_hard_light(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return select(
        2.0 * base * blend,
        1.0 - 2.0 * (1.0 - base) * (1.0 - blend),
        base <= vec3<f32>(0.5, 0.5, 0.5)
    );
}

// Blend mode enumeration
const BLEND_MODE_MULTIPLY: u32 = 0u;
const BLEND_MODE_SCREEN: u32 = 1u;
const BLEND_MODE_SOFT_LIGHT: u32 = 2u;
const BLEND_MODE_HARD_LIGHT: u32 = 3u;
const BLEND_MODE_OVERLAY: u32 = 4u;

// // Blend mode switch function
// fn blend_switch(mode: u32, base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
//     if (mode == BLEND_MODE_MULTIPLY) {
//         return blend_multiply(base, blend);
//     } else if (mode == BLEND_MODE_SCREEN) {
//         return blend_screen(base, blend);
//     } else if (mode == BLEND_MODE_SOFT_LIGHT) {
//         return blend_soft_light(base, blend);
//     } else if (mode == BLEND_MODE_HARD_LIGHT) {
//         return blend_hard_light(base, blend);
//     } else { // Assume overlay as the default mode
//         return blend_overlay(base, blend);
//     }
// }

// Blend mode switch function with blend amount
fn blend_switch(mode: u32, base: vec3<f32>, blend: vec3<f32>, blend_amount: f32) -> vec3<f32> {
    var blended_color: vec3<f32> = base;

    if (mode == BLEND_MODE_MULTIPLY) {
        blended_color = blend_multiply(base, blend);
    } else if (mode == BLEND_MODE_SCREEN) {
        blended_color = blend_screen(base, blend);
    } else if (mode == BLEND_MODE_SOFT_LIGHT) {
        blended_color = blend_soft_light(base, blend);
    } else if (mode == BLEND_MODE_HARD_LIGHT) {
        blended_color = blend_hard_light(base, blend);
    } else { // Assume overlay as the default mode
        blended_color = blend_overlay(base, blend);
    }

    return mix(base, blended_color, blend_amount);
}


@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(r_color, r_sampler, vertex.tex_coords);
    var rgb = color.rgb;

    let blend_mode = BLEND_MODE_SOFT_LIGHT;

    // Film Grain
    // let grain_scale = 1.0; // Controls the scale of the grain effect
    // let grain_intensity = 0.01; // Controls the intensity of the grain effect
    // let grain = perlin_noise(vertex.tex_coords * grain_scale + vec2<f32>(params.time, params.time));
    //
    // let grain_color = vec3<f32>(grain, grain, grain) * grain_intensity;
    // // rgb = blend_overlay(rgb, mix(vec3<f32>(0.5, 0.5, 0.5), grain_color, 0.001));
    // // rgb = blend_switch(blend_mode, rgb, mix(vec3<f32>(0.5, 0.5, 0.5), grain_color, params.film_grain_strength));
    //
    // rgb = blend_switch(blend_mode, rgb, grain_color, 0.1);
    // // rgb += mix(0.0, grain_intensity * (grain - 0.5), params.film_grain_strength);

    // Noise
    let noise_scale = 500.0; // Controls the scale of the noise effect
    let noise_intensity = 0.52; // Controls the intensity of the noise effect
    let noise = perlin_noise(vertex.tex_coords * noise_scale);

    let noise_color = vec3<f32>(noise, noise, noise) * noise_intensity;
    // rgb = blend_overlay(rgb, mix(vec3<f32>(0.5, 0.5, 0.5), noise_color, 0.001));
    // rgb = blend_switch(blend_mode, rgb, mix(vec3<f32>(0.5, 0.5, 0.5), noise_color, 0.005));
    rgb = blend_switch(blend_mode, rgb, noise_color, 0.5);

    // rgb += mix(0.0, noise_intensity * (noise - 0.5), params.noise_strength);

    // Vignetting
    let vignette_radius = 0.75; // Controls the size of the vignette effect
    let vignette_softness = 0.5; // Controls the softness of the vignette effect
    let vignette_center = vec2<f32>(0.5, 0.5);
    let vignette_dist = distance(vertex.tex_coords, vignette_center);
    let vignette_factor = smoothstep(vignette_radius, vignette_radius - vignette_softness, vignette_dist);

    rgb *= mix(1.0, vignette_factor, params.vignette_intensity);

    return vec4<f32>(rgb, color.a);
}

// @fragment
// fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
//     var color = textureSample(r_color, r_sampler, vertex.tex_coords);
//     var rgb = color.rgb;
//
//     // Film Grain
//     let grain_scale = 1.0; // Controls the scale of the grain effect
//     let grain_intensity = 0.1; // Controls the intensity of the grain effect
//     let grain_time_scale = 10.0; // Controls the speed of the grain effect
//     let grain = pseudo_rand(vertex.tex_coords * grain_scale + vec2<f32>(params.time * grain_time_scale, params.time * grain_time_scale));
//     rgb += mix(0.0, grain_intensity * (grain - 0.5), params.film_grain_strength);
//
//     // Noise
//     let noise_scale = 500.0; // Controls the scale of the noise effect
//     let noise_intensity = 0.1; // Controls the intensity of the noise effect
//     let noise = pseudo_rand(vertex.tex_coords * noise_scale);
//     rgb += mix(0.0, noise_intensity * (noise - 0.5), params.noise_strength);
//
//     // Vignetting
//     let vignette_radius = 0.75; // Controls the size of the vignette effect
//     let vignette_softness = 0.5; // Controls the softness of the vignette effect
//     let vignette_center = vec2<f32>(0.5, 0.5);
//     let vignette_dist = distance(vertex.tex_coords, vignette_center);
//     let vignette_factor = smoothstep(vignette_radius, vignette_radius - vignette_softness, vignette_dist);
//     rgb *= mix(1.0, vignette_factor, params.vignette_intensity);
//
//     return vec4<f32>(rgb, color.a);
// }

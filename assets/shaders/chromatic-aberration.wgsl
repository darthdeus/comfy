// @fragment
// fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
//     let uv: vec2<f32> = vertex.tex_coords;
//
//     let off: f32 = params.chromatic_aberration / params.resolution.x;
//
//     var color: vec3<f32>;
//
//     color.r = textureSample(r_color, r_sampler, vec2<f32>(uv.x + off, uv.y)).r;
//     color.g = textureSample(r_color, r_sampler, vec2<f32>(uv.x, uv.y)).g;
//     color.b = textureSample(r_color, r_sampler, vec2<f32>(uv.x - off, uv.y)).b;
//
//     return vec4<f32>(color, 1.0);
// }
//
// @fragment
// fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
//     let uv: vec2<f32> = vertex.tex_coords;
//
//     // Calculate the distance from the center of the screen
//     let dist_from_center: f32 = distance(uv, vec2<f32>(0.5, 0.5));
//
//     // Calculate the vignetting effect
//     let vignette_strength: f32 = smoothstep(0.0, 1.0, dist_from_center * 2.0);
//
//     // Adjust the chromatic aberration strength based on the vignetting effect
//     let off: f32 = params.chromatic_aberration * vignette_strength / params.resolution.x;
//
//     var color: vec3<f32>;
//
//     color.r = textureSample(r_color, r_sampler, vec2<f32>(uv.x + off, uv.y)).r;
//     color.g = textureSample(r_color, r_sampler, vec2<f32>(uv.x, uv.y)).g;
//     color.b = textureSample(r_color, r_sampler, vec2<f32>(uv.x - off, uv.y)).b;
//
//     return vec4<f32>(color, 1.0);
// }

// @fragment
// fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
//     let uv: vec2<f32> = vertex.tex_coords;
//
//     // Calculate the distance from the center of the screen
//     let dist_from_center: f32 = distance(uv, vec2<f32>(0.5, 0.5));
//
//     // Custom curve function
//     let custom_curve = 1.0 - pow(1.0 - dist_from_center, 3.0);
//
//     // Calculate the vignetting effect
//     let vignette_strength: f32 = custom_curve;
//
//     // Adjust the chromatic aberration strength based on the vignetting effect
//     let off: f32 = params.chromatic_aberration * vignette_strength / params.resolution.x;
//
//     var color: vec3<f32>;
//
//     color.r = textureSample(r_color, r_sampler, vec2<f32>(uv.x + off, uv.y)).r;
//     color.g = textureSample(r_color, r_sampler, vec2<f32>(uv.x, uv.y)).g;
//     color.b = textureSample(r_color, r_sampler, vec2<f32>(uv.x - off, uv.y)).b;
//
//     return vec4<f32>(color, 1.0);
// }

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let uv: vec2<f32> = vertex.tex_coords;

    // Calculate the distance from the center of the screen
    let dist_from_center: f32 = distance(uv, vec2<f32>(0.5, 0.5));

    // Custom curve function
    let custom_curve = 1.0 - pow(1.0 - dist_from_center, 3.0);

    // Calculate the vignetting effect
    let vignette_strength: f32 = custom_curve;

    // Add the center_boost parameter
    let center_boost: f32 = 0.2; // Change this value to control the minimum aberration at the center

    // Adjust the chromatic aberration strength based on the vignetting effect and center_boost
    let off: f32 = (params.chromatic_aberration * vignette_strength + center_boost * params.chromatic_aberration) / params.resolution.x;

    var color: vec3<f32>;

    color.r = textureSample(r_color, r_sampler, vec2<f32>(uv.x + off, uv.y)).r;
    color.g = textureSample(r_color, r_sampler, vec2<f32>(uv.x, uv.y)).g;
    color.b = textureSample(r_color, r_sampler, vec2<f32>(uv.x - off, uv.y)).b;

    return vec4<f32>(color, 1.0);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(r_color, r_sampler, vertex.tex_coords);
    return color;
}

// @fragment
// fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
//     var weight: array<f32, 5> = array<f32, 5>(
//         0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216
//     );
//
//
//     let tx = 1.0 / f32(textureDimensions(r_color).x);
//     let ty = 1.0 / f32(textureDimensions(r_color).y);
//
//     let tex_offset: vec2<f32> = vec2<f32>(tx, ty);
//     // let tex_offset: vec2<f32> = 1.0 / textureDimensions(r_color);
//
//     var result: vec3<f32> = textureSample(r_color, r_sampler, vertex.tex_coords).rgb * weight[0];
//
//     let horizontal = 1u;
//
//     if (horizontal == 1u) {
//         for (var i: u32 = 1u; i < 5u; i = i + 1u) {
//             result += textureSample(r_color, r_sampler, vertex.tex_coords + vec2<f32>(tex_offset.x * f32(i), 0.0)).rgb * weight[i];
//             result += textureSample(r_color, r_sampler, vertex.tex_coords - vec2<f32>(tex_offset.x * f32(i), 0.0)).rgb * weight[i];
//         }
//     } else {
//         for (var i: u32 = 1u; i < 5u; i = i + 1u) {
//             result += textureSample(r_color, r_sampler, vertex.tex_coords + vec2<f32>(0.0, tex_offset.y * f32(i))).rgb * weight[i];
//             result += textureSample(r_color, r_sampler, vertex.tex_coords - vec2<f32>(0.0, tex_offset.y * f32(i))).rgb * weight[i];
//         }
//     }
//
//     return vec4<f32>(result, 1.0);
// }

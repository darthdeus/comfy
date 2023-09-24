struct QuadUniform {
    clip_position: vec2<f32>,
    size: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> quad: QuadUniform;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var result: VertexOutput;

    var positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0,-1.0),
        vec2<f32>( 1.0,-1.0),

        vec2<f32>(-1.0, 1.0),
        vec2<f32>( 1.0,-1.0),
        vec2<f32>( 1.0, 1.0)
    );

    var tex_coords: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),

        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    result.position = vec4<f32>(
        positions[vertex_index] * quad.size + quad.clip_position,
        0.0,
        1.0
    );

    result.tex_coords = tex_coords[vertex_index];
    return result;
}

// @vertex
// fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
//     var result: VertexOutput;
//     // let x = i32(vertex_index) / 2;
//     // let y = i32(vertex_index) & 1;
//     let x = (i32(vertex_index) / 2) & 2;
//     let y = i32(vertex_index) & 1;
//     // let tc = vec2<f32>( (i32(vertex_index) / 2) & 2, i32(vertex_index) & 2 );
//
//     let tc = vec2<f32>(f32(x), f32(y));
//
//     result.position = vec4<f32>(
//         tc * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
//         0.0,
//         1.0
//     );
//
//     // let tc = vec2<f32>(f32(x), f32(y));
//
//     // result.position = vec4<f32>(
//     //     tc.x * 2.0 - 1.0,
//     //     1.0 - tc.y * 2.0,
//     //     0.0, 1.0
//     // );
//
//     // result.position = vec4<f32>(
//     //     quad.clip_position.x + (tc.x * quad.size.x),
//     //     quad.clip_position.y + (tc.y * quad.size.y),
//     //     0.0, 1.0
//     // );
//
//     result.tex_coords = tc;
//     return result;
// }


// @vertex
// fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
//     var result: VertexOutput;
//
//     // Convert the vertex index into a 2D index (i, j)
//     let i = i32(vertex_index % 3u);
//     let j = i32(vertex_index / 3u);
//
//     let tc = vec2<f32>(
//         f32(i % 2),
//         f32(j)
//     );
//
//     result.position = vec4<f32>(
//         quad.clip_position.x + (tc.x * quad.size.x),
//         quad.clip_position.y + (tc.y * quad.size.y),
//         0.0, 1.0
//     );
//
//     result.tex_coords = tc;
//     return result;
// }


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var r_color: texture_2d<f32>;
@group(0) @binding(1)
var r_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(r_color, r_sampler, vertex.tex_coords);

    // color.g = 0.0;
    // color.b = 0.0;

    return color;
}

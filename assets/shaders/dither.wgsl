@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(r_color, r_sampler, vertex.tex_coords);
    
    var randomValue = fract(sin(dot(vertex.tex_coords, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    
    if (randomValue < color.r) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}

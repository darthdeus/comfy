@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(r_color, r_sampler, vertex.tex_coords);

    var inverted_color = vec4<f32>(1.0, 1.0, 1.0, 1.0) - color;
    inverted_color.a = color.a;

    return inverted_color;
}

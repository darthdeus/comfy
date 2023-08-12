@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(r_color, r_sampler, vertex.tex_coords);

    color.r *= 1.1;
    color.g *= 0.9;
    color.b *= 0.9;

    color = clamp(
        color,
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        vec4<f32>(1.0, 1.0, 1.0, 1.0)
    );


    return color;
}

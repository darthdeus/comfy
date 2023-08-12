@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(r_color, r_sampler, vertex.tex_coords);
    
    color.g = 0.0;
    color.b = 0.0;

    return color;
}

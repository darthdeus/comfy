@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(r_color, r_sampler, vertex.tex_coords);

    let brightness = max(color.r, max(color.g, color.b));

    color = pow(color, vec4(params.bloom_gamma));

    if brightness > params.bloom_threshold {
        return color;
    } else {
        return color;
        // discard;
        // return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}

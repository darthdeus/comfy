// @group(0)
// @binding(2)
// var mipmap_level: u32;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the color from the specified mipmap level
    let mipmap_level = 4u;

    var color = textureSampleLevel(
        r_color,
        r_sampler,
        vertex.tex_coords,
        f32(mipmap_level)
    );

    // color.b = 1.0;
    
    return color;
}

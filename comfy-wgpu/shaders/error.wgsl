
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    var final_color: vec4<f32> = vec4<f32>(
      1.0,
      in.tex_coords.x,
      in.tex_coords.y,
      1.0
    );

    return final_color;
}

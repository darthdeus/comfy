@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var final_color: vec4<f32> = tex * in.color;

    // ***************************************************************
    // We can use our uniforms here directly by name. Their WGSL
    // declarations are automatically generated, mapped and checked
    // at runtime by Comfy.
    // ***************************************************************
    final_color.r = final_color.r * abs(cos(time * 3.0));
    final_color.g = final_color.g * abs(sin(time * 2.0));
    final_color.b = final_color.b * abs(cos(time * 5.0));

    final_color = final_color * intensity;

    return final_color;
}

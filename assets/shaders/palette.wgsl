@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex = textureSample(r_color, r_sampler, in.tex_coords);
    // tex.b = 1.0 - tex.b;

    let num_colors = 12;
    var colors = array<vec3<f32>, 12>(
        vec3<f32>(0.13, 0.07, 0.15),
        vec3<f32>(0.13, 0.08, 0.2),
        vec3<f32>(0.11, 0.12, 0.20),
        vec3<f32>(0.21, 0.36, 0.41),
        vec3<f32>(0.42, 0.69, 0.62),
        vec3<f32>(0.58, 0.77, 0.67),
        vec3<f32>(1.0, 0.92, 0.6),
        vec3<f32>(1.0, 0.76, 0.48),
        vec3<f32>(0.93, 0.6, 0.43),
        vec3<f32>(0.85, 0.38, 0.42),
        vec3<f32>(0.76, 0.29, 0.43),
        vec3<f32>(0.65, 0.19, 0.41),
    );

    var min_color = colors[0];
    var min_distance = distance(tex.rgb, colors[0]);

    for (var i = 0; i < num_colors; i++) {
        let d = distance(tex.rgb, colors[i]);

        if (d < min_distance) {
            min_distance = d;
            min_color = colors[i];
        }
    }

    // tex.r = min_color.r;
    // tex.g = min_color.g;
    // tex.b = min_color.b;

    // let gamma = 2.2;
    let gamma = 1.0;
    
    let replace = true;

    if replace {
        tex.r = min_color.r;
        tex.g = min_color.g;
        tex.b = min_color.b;
    }

    tex.r = pow(tex.r, gamma);
    tex.g = pow(tex.g, gamma);
    tex.b = pow(tex.b, gamma);

    // tex.xyz = pow(tex.xyz, vec3<f32>(1.0/gamma));

    // if (replace) {
        // tex.rgb = min_color.rgb;
    // }

    return tex;
    // return vec4<f32>(1.0, 0.0, 0.5, 1.0);
}

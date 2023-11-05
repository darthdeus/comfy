fn apply_light(in: VertexOutput, light: Light) -> vec4<f32> {
    let light_to_frag = in.world_position.xy - light.world_position;
    let distance = length(light_to_frag);

    if (distance > light.radius) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let attenuation = 1.0 / (1.0 + light.radius * distance * distance);
    let falloff = 1.0 - (distance / light.radius);

    let modifier = select(attenuation, falloff, params.quadratic_falloff == 0u);
    var diffuse = light.strength * modifier;

    return light.color * diffuse * params.global_light_intensity;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let base_color: vec4<f32> = tex * in.color;

    // Ambient lighting
    var ambient_color: vec4<f32> = params.ambient_light_color * params.ambient_light_intensity;
    var final_color: vec4<f32> = base_color * ambient_color;

    if (params.lighting_enabled == 1u) {
        for (var i: u32 = 0u; i < lights.light_count; i = i + 1u) {
            let light = lights.lights[i];
            final_color = final_color + (base_color * apply_light(in, light));
        }
    }

    // Apply gamma correction
    final_color.r = pow(final_color.r, params.gamma_correction);
    final_color.g = pow(final_color.g, params.gamma_correction);
    final_color.b = pow(final_color.b, params.gamma_correction);
    final_color.a = base_color.a;

    return final_color;
}

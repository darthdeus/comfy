@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let ratio = params.resolution.x / params.resolution.y;
    let scale_val = 1920.0;
    let scale = vec2<f32>(scale_val, scale_val / ratio);
    let uv = scale / round(vertex.tex_coords * scale);

    var color = textureSample(r_color, r_sampler, vertex.tex_coords);

    let noise = color.r * color.r * params.time * 0.01;
    // let noise = color.r * color.r * color.r * params.time * 0.0001;
    
    // var randomValue = fract(sin(dot(uv + vec2<f32>(noise), vec2<f32>(12.9898, 78.233))) * 43758.5453);
    var randomValue = fract(sin(dot(uv + vec2<f32>(uv), vec2<f32>(12.9898, 78.233))) * (43758.5453 + noise));

    let dither_enabled = true;

    // return vec4<f32>(randomValue, randomValue, randomValue, 1.0);

    if dither_enabled {
      if (randomValue < color.r) {
          return vec4<f32>(1.0, 1.0, 1.0, 1.0);
      } else {
          return vec4<f32>(0.0, 0.0, 0.0, 1.0);
      }
    } else {
        return color;
    }
}

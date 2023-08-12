// @fragment
// fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
//     // Generate a random value based on the texture coordinates and time
//     var random_value = fract(sin(dot(vertex.tex_coords, vec2<f32>(12.9898, 78.233))) * 43758.5453 + params.time);
//
//     // Calculate the screen shake offset
//     var shake_offset = params.shake_amount * (random_value - 0.5);
//
//     // Displace the texture coordinates with the screen shake offset
//     var displaced_tex_coords = vertex.tex_coords + shake_offset;
//
//     // Sample the texture using the displaced texture coordinates
//     var color = textureSample(r_color, r_sampler, displaced_tex_coords);
//
//     return color;
// }

fn pseudo_rand(v: vec2<f32>) -> f32 {
    return fract(sin(dot(v, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate a random value based on the texture coordinates
    var random_value = pseudo_rand(vertex.tex_coords * vec2<f32>(1000.0, 1000.0));

    let freq = 10.0;
    let frequency = vec2<f32>(freq, freq);
    let speed = 1.0;

    // Calculate the screen shake offset using two sine waves with different frequencies and the random value
    var shake_offset = params.shake_amount * sin(frequency * params.time + vec2<f32>(0.0, 3.14159) * random_value) * speed;

    // Displace the texture coordinates with the screen shake offset
    var displaced_tex_coords = vertex.tex_coords + shake_offset;

    // Sample the texture using the displaced texture coordinates
    var color = textureSample(r_color, r_sampler, displaced_tex_coords);

    return color;
}

// @fragment
// fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
//     // Remove the random factor and use a constant seed value
//     let seed = vec2<f32>(12.34, 56.78);
//
//     let freq = 10.0;
//     let frequency = vec2<f32>(freq, freq);
//     let speed = 1.0;
//
//     // Calculate the screen shake offset using two sine waves with different frequencies
//     var shake_offset = params.shake_amount * sin(frequency * params.time + seed) * speed;
//
//     // Displace the texture coordinates with the screen shake offset
//     var displaced_tex_coords = vertex.tex_coords + shake_offset;
//
//     // Sample the texture using the displaced texture coordinates
//     var color = textureSample(r_color, r_sampler, displaced_tex_coords);
//
//     return color;
// }


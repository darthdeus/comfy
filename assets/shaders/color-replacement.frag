#version 100
precision mediump float;

varying vec4 color;
varying vec2 uv;
varying vec2 worldPosition;

uniform sampler2D Texture;

uniform vec3 Color1;
uniform vec3 Color2;
uniform vec3 Color3;
uniform vec3 Color4;
uniform vec3 Color5;
uniform vec3 Color6;
uniform vec3 Color7;
uniform vec3 Color8;

float dist(vec3 a, vec3 b) {
  return distance(a, b);
  // return dot(a, b) / (length(a) * length(b));
}

void main() {
    vec4 res = texture2D(Texture, uv);

    bool replace = true;
    int num_colors = 12;

    vec3 colors[12];

    // colors[0] = Color1;
    // colors[1] = Color2;
    // colors[2] = Color3;
    // colors[3] = Color4;
    // colors[4] = Color5;
    // colors[5] = Color6;
    // colors[6] = Color7;
    // colors[7] = Color8;

    // https://lospec.com/palette-list/berry-nebula
    // colors[0] = vec3(0.43, 0.52, 0.65);
    // colors[1] = vec3(0.42, 0.73, 0.79);
    // colors[2] = vec3(0.42, 0.93, 0.93);
    // colors[3] = vec3(0.43, 0.32, 0.51);
    // colors[4] = vec3(0.44, 0.11, 0.36);
    // colors[5] = vec3(0.31, 0.08, 0.27);
    // colors[6] = vec3(0.18, 0.04, 0.19);
    // colors[7] = vec3(0.05, 0.0, 0.1);

    // https://lospec.com/palette-list/akc12
    colors[0] = vec3(0.13, 0.07, 0.15);
    colors[1] = vec3(0.13, 0.08, 0.2);
    colors[2] = vec3(0.11, 0.12, 0.20);
    colors[3] = vec3(0.21, 0.36, 0.41);
    colors[4] = vec3(0.42, 0.69, 0.62);
    colors[5] = vec3(0.58, 0.77, 0.67);
    colors[6] = vec3(1.0, 0.92, 0.6);
    colors[7] = vec3(1.0, 0.76, 0.48);
    colors[8] = vec3(0.93, 0.6, 0.43);
    colors[9] = vec3(0.85, 0.38, 0.42);
    colors[10] = vec3(0.76, 0.29, 0.43);
    colors[11] = vec3(0.65, 0.19, 0.41);

    vec3 min_color = colors[0];
    // THIS WORKS
    float min_distance = dist(res.rgb, colors[0]);
    // THIS IS TOO SMALL???
    // float min_distance = 9999999999.0;

    for (int i = 0; i < num_colors; i++) {
        float d = dist(res.rgb, colors[i]);

        if (d < min_distance) {
            min_distance = d;
            min_color = colors[i];
        }
    }

    if (replace) {
        res.rgb = min_color.rgb;
    }

    // res.r -= 0.3;

    gl_FragColor = res;
}

#version 100
precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

const float A = 2.51;
const float B = 0.03;
const float C = 2.43;
const float D = 0.59;
const float E = 0.14;

void main() {
  // vec4 pixel = texture2D(Texture, gl_FragCoord.xy / textureSize(Texture, 0));
  vec4 pixel = texture2D(Texture, uv);

  // Calculate the ACES tonemapping curve value for this pixel
  float x = max(0.0, pixel.r);
  float curveValue = (x*(A*x+B))/(x*(C*x+D)+E);

  // Apply the tonemapping curve to the pixel color
  vec3 color = curveValue * pixel.rgb;

  // color = pixel.rgb;

  gl_FragColor = vec4(color, 1.0);
}

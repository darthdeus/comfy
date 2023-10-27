out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D scene;
uniform sampler2D bloomBlur;

uniform float exposure;
uniform float bloomGamma;
uniform float colorScale;

uniform int tonemapping_alg;

// Narkowicz 2015, "ACES Filmic Tone Mapping Curve"
vec3 aces(vec3 x) {
  const float a = 2.51;
  const float b = 0.03;
  const float c = 2.43;
  const float d = 0.59;
  const float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

float aces(float x) {
  const float a = 2.51;
  const float b = 0.03;
  const float c = 2.43;
  const float d = 0.59;
  const float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// Filmic Tonemapping Operators http://filmicworlds.com/blog/filmic-tonemapping-operators/
vec3 filmic(vec3 x) {
  vec3 X = max(vec3(0.0), x - 0.004);
  vec3 result = (X * (6.2 * X + 0.5)) / (X * (6.2 * X + 1.7) + 0.06);
  return pow(result, vec3(2.2));
}

float filmic(float x) {
  float X = max(0.0, x - 0.004);
  float result = (X * (6.2 * X + 0.5)) / (X * (6.2 * X + 1.7) + 0.06);
  return pow(result, 2.2);
}

void main() {
  vec3 hdrColor = texture(scene, TexCoords).rgb;
  vec3 bloomColor = texture(bloomBlur, TexCoords).rgb;

  // additive blending
  hdrColor *= 1.0;
  hdrColor += 1.0 * bloomColor;

  // tone mapping
  vec3 result;

  if (tonemapping_alg == 0) {
    result = vec3(1.0) - exp(-hdrColor * exposure);
  } else if (tonemapping_alg == 1) {
    result = filmic(hdrColor);
  } else if (tonemapping_alg == 2) {
    result = aces(hdrColor);
  } else if (tonemapping_alg == 3) {
    result = hdrColor;
  } else {
    result = vec3(1.0) - exp(-hdrColor * exposure);
    result.gb = vec2(0.0);
  }

  // gamma correction
  result = pow(result, vec3(1.0 / bloomGamma));
  FragColor = vec4(result, 1.0);

  // FragColor = vec4(bloomColor, 1.0);
}

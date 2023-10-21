out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D image;
uniform bool horizontal;

void main() {

#define NUM 5
  float weight[5] = float[] (
    0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216
  );

    // 0.210458, 0.170794, 0.144487, 0.120098, 0.102544,
    // 0.072534, 0.06226, 0.049012, 0.032542, 0.019000,
    // 0.009815, 0.004303, 0.001562, 0.000460, 0.000109,
    // 0.000015, 0.000003, 0.000002, 0.000002, 0.000001

// #define NUM 20
//   float weight[20] = float[] (
// 0.105229, 0.085397, 0.072243, 0.060049, 0.051272,
// 0.036267, 0.03113, 0.024506, 0.016271, 0.009500,
// 0.0049075, 0.0021515, 0.000781, 0.00023, 0.000055,
// 0.0000075, 0.0000015, 0.000001, 0.000001, 0.0000005  
// );

  vec2 tex_offset = 1.0 / textureSize(image, 0);
  vec3 result = texture(image, TexCoords).rgb * weight[0];

  if (horizontal) {
    for (int i = 1; i < NUM; ++i) {
      result += texture(image, TexCoords + vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
      result += texture(image, TexCoords - vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
    }
  } else {
    for (int i = 1; i < NUM; ++i) {
      result += texture(image, TexCoords + vec2(0.0, tex_offset.y * i)).rgb * weight[i];
      result += texture(image, TexCoords - vec2(0.0, tex_offset.y * i)).rgb * weight[i];
    }
  }

  // result.rgb = clamp(FragColor.rgb, 0.0, 1.0);

  FragColor = vec4(result, 1.0);
}

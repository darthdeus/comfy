in vec2 TexCoords;
layout(location = 0) out vec4 color;

uniform vec3 iResolution;
uniform vec4 iMouse;
uniform float iTime;
uniform float iTimeDelta;
uniform int iFrame;
uniform float iFrameRate;

uniform sampler2D iChannel0;
uniform sampler2D iChannel1;
uniform sampler2D iChannel2;
uniform sampler2D iChannel3;

uniform vec2 offsets[9];
uniform int edge_kernel[9];
uniform float blur_kernel[9];

uniform bool chaos;
uniform bool confuse;
uniform bool shake;
uniform bool skip_pp;

vec4 input_color;

vec4 shake_func() {
  vec4 result = vec4(0.0f);
  vec3 sample[9];

  // sample from texture offsets if using convolution matrix
  if (chaos || shake) {
    for (int i = 0; i < 9; i++) {
      sample[i] = vec3(texture(iChannel0, TexCoords.st + offsets[i]));
    }
  }

  // process effects
  if (chaos) {
    for (int i = 0; i < 9; i++) {
      result += vec4(sample[i] * edge_kernel[i], 0.0f);
    }

    result.a = 1.0f;
  } else if (confuse) {
    result = vec4(1.0 - texture(iChannel0, TexCoords).rgb, 1.0);
  } else if (shake) {
    for (int i = 0; i < 9; i++) {
      result += vec4(sample[i] * blur_kernel[i], 0.0f);
    }

    result.a = 1.0f;
  } else {
    result = texture(iChannel0, TexCoords);
  }

  return result;
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
  vec4 sample0 = texture(iChannel0, fragCoord / iResolution.xy);
  vec4 sample1 =
      texture(iChannel0, (fragCoord + vec2(1.0, 1.0)) / iResolution.xy);
  vec4 sample2 =
      texture(iChannel0, (fragCoord + vec2(-1.0, -1.0)) / iResolution.xy);
  vec4 sample3 =
      texture(iChannel0, (fragCoord + vec2(-1.0, 1.0)) / iResolution.xy);
  vec4 sample4 =
      texture(iChannel0, (fragCoord + vec2(1.0, -1.0)) / iResolution.xy);

  float edge = checkSame(sample1, sample2) * checkSame(sample3, sample4);

  fragColor = vec4(edge, sample0.w, 1.0, 1.0);
}

/////////////////////////////////////////////////////

void main() {
  input_color = texture(iChannel0, TexCoords);
  
  if (skip_pp) {
    color = input_color;
  } else {
    input_color = shake_func();
    vec4 gen_color = vec4(1.0);

    mainImage(gen_color, gl_FragCoord.xy);

    color = vec4(vec3(1.0).xyz - mix(input_color, gen_color, 1.0).xyz, 1.0);
    // color = input_color;
  }
}

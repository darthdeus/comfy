layout (location = 0) out vec4 FragColor;

in vec3 pos;
in vec4 vertexColor;
in vec2 TexCoords;
in vec2 worldPos;

uniform sampler2D texture0;

uniform float bloomThreshold;
uniform float colorScale;

float exponentialIn(float t) {
  return t == 0.0 ? t : pow(2.0, 10.0 * (t - 1.0));
}

float quadraticIn(float t) {
  return t * t;
}

float quarticIn(float t) {
  return pow(t, 4.0);
}

float qinticIn(float t) {
  return pow(t, 5.0);
}

struct Light {
  vec4 color;
  vec2 world_position;
  vec2 screen_position;
  float radius;
  float strength;
};

#define MAX_NUM_TOTAL_LIGHTS 128

layout (std140) uniform Lights {
  Light lights[MAX_NUM_TOTAL_LIGHTS];
  int num_lights;
  // TODO: missing padding
};

void main() {
  FragColor = texture(texture0, TexCoords) * vertexColor;

  float mult = 1.0;

  for (int i = 0; i < num_lights; i++) {
    float d = distance(lights[i].world_position, worldPos);

    float radius = lights[i].radius;
    float pct = clamp(distance(lights[i].world_position, worldPos) / radius, 0.0, 1.0);
    float t = 1.0 - pct;
    mult += lights[i].strength * t * t;

  }

  FragColor.rgb *= vec3(mult);

    // float dist = distance(lights[i].screen_position, gl_FragCoord.xy);
    // float d = dist / 1920.0;
    // float max_dist = 20.0;
    // float new_mult = clamp(1.0 / (max_dist * d), 0.0, 2.0);
    // float new_mult = clamp(
    //   lights[i].radius / (3.0 * d),
    //   0.0,
    //   lights[i].strength
    // );

    // float d = distance(lights[i].world_position, worldPos);
    // float max_dist = 2.0;
    // float new_mult = clamp(20.0 / (max_dist * d), 0.0, 2.0);
    // if (d > lights[i].radius) {
    //   new_mult = 0.0;
    // }

    // mult += lights[i].strength * exponentialIn(t);

  // .... Working test
  // float radius = lights[0].radius;
  // float pct = clamp(distance(vec2(0.0, 0.0), worldPos) / radius, 0.0, 1.0);
  // float t = 1.0 - pct;
  //
  // FragColor.rgb = vec3(0.0, 0.0, 0.0);
  // FragColor.r = quarticIn(t);
  // ^^^^^^^^^^^^^^^

  if (FragColor.a < 0.01) {
    discard;
  }
  // }
}

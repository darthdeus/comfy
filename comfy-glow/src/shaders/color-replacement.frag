out vec4 FragColor;

in vec3 pos;
in vec2 TexCoords;

uniform sampler2D texture0;

uniform bool replacePalette = true;
uniform vec3 palette[8];

void main() {
  FragColor = texture(texture0, TexCoords);

  vec3 colors[12] = vec3[](
    vec3(0.13, 0.07, 0.15),
    vec3(0.13, 0.08, 0.2),
    vec3(0.11, 0.12, 0.20),
    vec3(0.21, 0.36, 0.41),
    vec3(0.42, 0.69, 0.62),
    vec3(0.58, 0.77, 0.67),
    vec3(1.0, 0.92, 0.6),
    vec3(1.0, 0.76, 0.48),
    vec3(0.93, 0.6, 0.43),
    vec3(0.85, 0.38, 0.42),
    vec3(0.76, 0.29, 0.43),
    vec3(0.65, 0.19, 0.41)
  );

  float distances[8];

  for (int i = 0; i < 8; i++) {
    distances[i] = distance(colors[i], FragColor.xyz);
  }

  int minIndex = 0;
  float minDistance = distances[0];

  for (int i = 0; i < 8; i++) {
    if (distances[i] < minDistance) {
      minIndex = i;
      minDistance = distances[i];
    }
  }

  if (replacePalette) {
    FragColor = vec4(colors[minIndex], FragColor.a);
  }

  // if (FragColor.a < 0.1) {
  //   discard;
  // }

  /* FragColor = texture(texture0, TexCoords) * tint; */
  /* FragColor = vec4(vertexColor, 1.0); */
  /* FragColor = vertexColor; */
}

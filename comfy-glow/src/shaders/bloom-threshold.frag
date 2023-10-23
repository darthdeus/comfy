layout (location = 0) out vec4 FragColor;

in vec3 pos;
in vec4 vertexColor;
in vec2 TexCoords;

uniform sampler2D texture0;

uniform float bloomThreshold;
uniform float colorScale;

void main() {
  FragColor = texture(texture0, TexCoords);
  FragColor.rgb *= colorScale;

  float brightness = dot(FragColor.rgb, vec3(0.2126, 0.7152, 0.0722));

  if (brightness > bloomThreshold) {
    FragColor = vec4(FragColor.rgb, 1.0);
  } else {
    FragColor = vec4(0.0, 0.0, 0.0, FragColor.a);
  }
}

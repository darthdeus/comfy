layout (location = 0) out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D texture0;

void main() {
  FragColor = texture(texture0, TexCoords);
}

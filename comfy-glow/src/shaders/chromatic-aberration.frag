out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D image;

uniform float chromatic_aberration = 5.0;
uniform vec2 iResolution;

void main() {
  vec2 uv = TexCoords;

  float off = chromatic_aberration / float(iResolution.x);

  vec3 color;

  color.r = texture(image, vec2(uv.x + off, uv.y)).r;
  color.g = texture(image, vec2(uv.x, uv.y)).g;
  color.b = texture(image, vec2(uv.x - off, uv.y)).b;

  FragColor = vec4(color, 1.0);
}

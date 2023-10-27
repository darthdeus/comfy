out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D image;

uniform float contrast = 1.0;
uniform float brightness = 0.0;
uniform float saturation = 1.0;
uniform float gamma = 1.0;

void main() {
  vec3 color = texture(image, TexCoords).rgb;

  color = clamp(contrast * (color - 0.5) + 0.5 + brightness, 0.0, 1.0);
  
  float grayscale = dot(color, vec3(0.299, 0.587, 0.114));
  color.r = clamp(mix(grayscale, color.r, saturation), 0.0, 1.0);
  color.g = clamp(mix(grayscale, color.g, saturation), 0.0, 1.0);
  color.b = clamp(mix(grayscale, color.b, saturation), 0.0, 1.0);

  color.r = clamp(pow(color.r, gamma), 0.0, 1.0);
  color.g = clamp(pow(color.g, gamma), 0.0, 1.0);
  color.b = clamp(pow(color.b, gamma), 0.0, 1.0);

  FragColor = vec4(color, 1.0);
}

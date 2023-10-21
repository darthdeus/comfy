// layout (location = 0) in vec4 vertex; // <vec2 position, vec2 texCoords>
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec2 aTexCoords;

out vec2 TexCoords;

uniform vec3 iResolution;
uniform vec4 iMouse;
uniform float iTime;
uniform float iTimeDelta;
uniform int iFrame;
uniform float iFrameRate;

uniform bool chaos;
uniform bool confuse;
uniform bool shake;

uniform float shake_amount = 1.0;

void main() {
  gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0f);
  vec2 texture = aTexCoords;

  if (chaos) {
    float strength = 0.3;
    vec2 pos = vec2(texture.x + sin(iTime) * strength,
                    texture.y + cos(iTime) * strength);
    TexCoords = pos;
  } else if (confuse) {
    TexCoords = vec2(1.0 - texture.x, 1.0 - texture.y);
  } else {
    TexCoords = texture;
  }

  if (shake) {
    float strength = shake_amount * 0.01;
    gl_Position.x += cos(iTime * 10) * strength;
    gl_Position.y += cos(iTime * 15) * strength;
  }
}

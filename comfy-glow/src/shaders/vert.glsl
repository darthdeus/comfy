layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec4 aColor;

out vec3 pos;
out vec4 vertexColor;
out vec2 TexCoords;
out vec2 worldPos;

// uniform mat4 model;
// uniform mat4 view;
uniform mat4 projection;

void main() {
  // gl_Position = projection * view * model * vec4(aPos, 1.0);
  gl_Position = projection * vec4(aPos, 1.0);
  worldPos = aPos.xy;
  pos = gl_Position.xyz;
  vertexColor = aColor;
  TexCoords = aTexCoord;
  // gl_Position = vec4(aPos.xyz, 0.0);
}

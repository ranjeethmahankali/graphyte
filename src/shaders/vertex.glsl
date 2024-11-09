#version 410

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;

out vec4 vertNorm;
out vec4 vertNormWorld;
out vec3 vertColor;

uniform mat4 u_mvp;

void main()
{
  gl_Position = u_mvp * vec4(position, 1.0);
  vertNorm      = normalize(u_mvp * vec4(normal, 0.0));
  vertNormWorld = normalize(vertNorm);
  vertColor     = color;
}

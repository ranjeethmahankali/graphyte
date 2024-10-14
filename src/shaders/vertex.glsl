#version 330

const vec3 verts[9] = vec3[9](vec3(1.0, 0.0, 0.0),
                              vec3(0.0, 1.0, 0.0),
                              vec3(0.0, 0.0, 0.0),
                              vec3(0.0, 1.0, 0.0),
                              vec3(0.0, 0.0, 1.0),
                              vec3(0.0, 0.0, 0.0),
                              vec3(0.0, 0.0, 1.0),
                              vec3(1.0, 0.0, 0.0),
                              vec3(0.0, 0.0, 0.0));
const vec4 colors[3] = vec4[3](vec4(1.0, 0.0, 0.0, 1.0),
                               vec4(0.0, 1.0, 0.0, 1.0),
                               vec4(0.0, 0.0, 1.0, 1.0));
out vec4 v_color;
uniform mat4 u_mvp;
void main() {
  v_color = colors[gl_VertexID / 3];
  gl_Position = u_mvp * vec4(verts[gl_VertexID], 1.0);
}

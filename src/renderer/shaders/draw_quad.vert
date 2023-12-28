#version 460

layout(location = 0) in vec2 a_position; // Vertex position
layout(location = 1) in vec2 a_uv;       // Vertex UV

layout(location = 0) out vec2 v_uv; // Output UV to fragment shader

void main() {
  v_uv = a_uv;
  gl_Position = vec4(a_position, 0.0, 1.0);
}

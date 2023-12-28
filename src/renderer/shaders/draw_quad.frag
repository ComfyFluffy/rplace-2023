#version 460

layout(location = 0) out vec4 fragColor; // Output color

layout(location = 0) in vec2 v_uv; // Input UV from vertex shader

layout(binding = 0) uniform sampler2D u_myTexture; // Texture uniform

void main() {
    fragColor = texture(u_myTexture, v_uv);
}

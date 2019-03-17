#version 450

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in vec2 tex_uv;

layout(location = 0) out vec4 color;

void main() {
    color = texture(albedo, tex_uv);
}

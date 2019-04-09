#version 450

layout(push_constant) uniform pushConstants {
    uint tex_id;
} u_pushConstants;

layout(set = 1, binding = 0) uniform sampler2D albedo[32];

layout(location = 0) in vec2 tex_uv;

layout(location = 0) out vec4 color;

void main() {
    color = texture(albedo[u_pushConstants.tex_id], tex_uv);
}

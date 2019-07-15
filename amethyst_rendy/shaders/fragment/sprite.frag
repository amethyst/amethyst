#version 450

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec2 tex_uv;
    vec4 color;
} vertex;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(albedo, vertex.tex_uv) * vertex.color;
    if (color.a == 0.0) {
        discard;
    }
    out_color = color;
}

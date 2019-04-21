#version 450

layout(set = 1, binding = 0) uniform sampler2D tex;

layout(location = 0) in VertexData {
    // TODO(happens): See vertex shader, is this still needed for something?
    vec4 pos;
    vec2 tex_uv;
    vec4 color;
} vertex;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(tex, vertex.tex_uv) * vertex.color;
    if (color.a == 0.0) {
        discard;
    }

    out_color = color;
}

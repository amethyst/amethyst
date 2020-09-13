#version 450

layout(set = 1, binding = 0) uniform sampler2D albedo1;
layout(set = 2, binding = 0) uniform sampler2D albedo2;

layout(location = 0) in VertexData {
    vec2 tex_uv;
    vec4 color;
} vertex;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 color1 = texture(albedo1, vertex.tex_uv);
    vec4 color2 = texture(albedo2, vertex.tex_uv);

     if (color1.a == 0.0 || color2.a == 0.0) {
        discard;
    }

    out_color = color1 +color2 /2.0;
}

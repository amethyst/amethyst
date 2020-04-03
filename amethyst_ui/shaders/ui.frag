#version 450

layout(set = 1, binding = 0) uniform sampler2D tex;

layout(location = 0) in vec2 vert_out_tex_coords;
layout(location = 1) in vec4 vert_out_color;
layout(location = 2) in vec4 vert_out_color_bias;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = (texture(tex, vert_out_tex_coords) + vert_out_color_bias) * vert_out_color;
    if (color.a == 0.0) {
        discard;
    }

    out_color = color;
}

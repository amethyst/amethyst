#version 450

// TODO(happens): should these be in the same set?
// decide after implementing drawing
layout(set = 1, binding = 0) uniform sampler2D tex;
layout(set = 2, binding = 0) uniform vec4 color;

layout(location = 0) in vec2 tex_uv;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 color_result = texture(tex, tex_uv) * color;
    if (color_result.a == 0.0) {
        discard;
    }

    out_color = color_result;
}

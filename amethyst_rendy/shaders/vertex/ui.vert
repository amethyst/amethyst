#version 450

layout(std140, set = 0, binding = 0) uniform UiViewArgs {
    uniform vec2 inverse_window_size;
};

layout(location = 0) in vec2 in_coords;
layout(location = 1) in vec2 in_dimensions;
layout(location = 2) in vec4 in_color;

layout(location = 0) out vec2 out_tex_coords;
layout(location = 1) out vec4 out_color;

void main() {
    vec4 proj = vec4(inverse_window_size, 1, 1);
    vec4 pos = vec4(-1, -1, 0, 1);

    // TODO(happens): Clean up this mess
    pos *= vec4(in_dimensions, 1, 1);
    pos *= proj;
    pos += vec4(in_coords * 2, 0, 0) * proj;
    pos += vec4(-1, -1, 0, 0);

    out_tex_coords = vec2(0, 0);
    out_color = in_color;

    gl_Position = pos;
}

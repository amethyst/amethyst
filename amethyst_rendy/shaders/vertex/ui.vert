#version 450

layout(std140, set = 0, binding = 0) uniform UiViewArgs {
    uniform vec2 inverse_window_size;
};

layout(location = 0) in vec2 in_coords;
layout(location = 1) in vec2 in_dimensions;
layout(location = 2) in vec4 in_color;

layout(location = 0) out vec2 out_tex_coords;
layout(location = 1) out vec4 out_color;

const vec2 positions[6] = vec2[](
    // First triangle
    vec2(-0.5, -0.5), // Left bottom
    vec2(0.5, -0.5), // Right bottom
    vec2(0.5, 0.5), // Right top

    // Second triangle
    vec2(0.5, 0.5), // Right top
    vec2(-0.5, 0.5), // Left top
    vec2(-0.5, -0.5)  // Left bottom
);

void main() {
    vec2 pos = positions[gl_VertexIndex];

    out_tex_coords = vec2(pos.x+0.5, pos.y+0.5);
    out_color = in_color;

    float pos_x = (pos.x * inverse_window_size.x * 2) - 1;
    float pos_y = (pos.y * inverse_window_size.y * 2) - 1;
    gl_Position = vec4(pos_x, pos_y, 1, 1);
}

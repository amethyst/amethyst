#version 450

layout(std140, set = 0, binding = 0) uniform UiViewArgs {
    uniform vec2 inverse_window_size;
};

layout(location = 0) in vec2 in_coords;
layout(location = 1) in vec2 in_dimensions;
layout(location = 2) in vec4 in_color;

layout(location = 0) out vec2 out_tex_coords;
layout(location = 1) out vec4 out_color;

const vec2 positions[4] = vec2[](
    vec2(0.5, -0.5), // Right bottom
    vec2(-0.5, -0.5), // Left bottom
    vec2(0.5, 0.5), // Right top
    vec2(-0.5, 0.5) // Left top
);

void main() {
    vec2 pos = positions[gl_VertexIndex];

    out_tex_coords = vec2(pos.x+0.5, pos.y+0.5);
    out_color = in_color;

    vec2 center = in_coords * inverse_window_size;
    center.y = 1.0 - center.y; 
    vec2 final_pos = (center + in_dimensions * inverse_window_size * pos) * 2.0 - vec2(1.0);

    gl_Position = vec4(final_pos, 0.0, 1.0);
}
#version 450

// TODO(hilmar): Not all of these are changing, would it be beneficial
// to split this up into multiple sets?
layout(std140, set = 0, binding = 0) uniform UiViewArgs {
    uniform vec2 inverse_window_size;
    uniform vec2 coords;
    uniform vec2 dimensions;
};

// Square [-1.0,1.0]
layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex_coord;

layout(location = 0) out vec2 tex_uv;

void main() {
    vec4 window_proj = vec4(inverse_window_size, 1, 1);
    vec4 pos = vec4(pos, 1);

    // scale the element
    pos *= vec4(dimensions, 1, 1);

    // position is now in pixel coordinates, normalize it
    pos *= window_proj;
    pos += vec4(coords * 2, 0, 0) * window_proj;

    // recenter viewport
    pos += vec4(-1, -1, 0, 0);

    tex_uv = tex_coord;
    gl_Position = pos;
}

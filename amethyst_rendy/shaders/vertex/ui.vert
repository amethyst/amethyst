#version 450

// TODO(hilmar): Not all of these are changing, would it be beneficial
// to split this up into multiple structs?
layout(std140, set = 0, binding = 0) uniform UiVertexArgs {
    uniform vec2 inverse_window_size;
    uniform vec2 coords;
    uniform vec2 dimensions;
    uniform vec4 color;
};

// Square [-1.0,1.0]
layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex_uv;

layout(location = 0) out VertexData {
    // TODO(happens): Why is this passed along? Pos doesn't seem
    // to be used at all in the original UI frag shader
    vec4 pos;
    vec2 tex_uv;
    vec4 color;
} vertex;

void main() {
    vec4 window_proj = vec4(inverse_window_size, 1, 1);
    vertex.position = vec4(pos, 1);

    // scale the element
    vertex.pos *= vec4(dimensions, 1, 1);

    // position is now in pixel coordinates, normalize it
    vertex.pos *= window_proj;
    vertex.pos += vec4(coords * 2, 0, 0) * window_proj;

    // recenter viewport
    vertex.pos += vec4(-1, -1, 0, 0);

    vertex.tex_uv = tex_coord;
    vertex.color = color;
    gl_Position = vertex.pos;
}

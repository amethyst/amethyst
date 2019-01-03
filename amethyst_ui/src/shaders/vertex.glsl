// TODO: Needs documentation.

#version 150 core

// std140 is a cross platform layout.
layout (std140) uniform VertexArgs {
    uniform vec2 invert_window_size;
    uniform vec2 coord;
    uniform vec2 dimension;
    uniform vec4 color;
};

// Square [-1.0,1.0]
in vec3 position;
in vec2 tex_coord;

out VertexData {
  vec4 position;
  vec2 tex_coord;
  vec4 color;
} vertex;

void main() {
    vec4 proj_vec4 = vec4(invert_window_size, 1, 1);
    // Create a vec4 with w=1
    vertex.position = vec4(position, 1);

    // Multiply by the size of the element we want to draw.
    // [-elem_width, +elem_width]
    vertex.position *= vec4(dimension, 1, 1);

    // Scale everything back down from pixel coordinates to [-1,1] domain.
    vertex.position *= proj_vec4;

    // Move everything by the coordinates of the element.
    vertex.position += vec4(coord * 2, 0, 0) * proj_vec4;

    // Recenter the whole viewport.
    vertex.position += vec4(-1, -1, 0, 0);

    vertex.tex_coord = tex_coord;
    vertex.color = color;
    gl_Position = vertex.position;
}

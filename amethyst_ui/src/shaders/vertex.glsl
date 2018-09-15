// TODO: Needs documentation.

#version 150 core

// std140 is a cross platform layout.
layout (std140) uniform VertexArgs {
    uniform vec4 proj_vec;
    uniform vec2 coord;
    uniform vec2 dimension;
};

// Square [-0.5,0.5]
in vec3 position;
in vec2 tex_coord;

out VertexData {
  vec4 position;
  vec2 tex_coord;
} vertex;

void main() {
    // Create a vec4 with w=1
    vertex.position = vec4(position, 1);

    // Multiply by the size of the element we want to draw.
    // [-elem_width, +elem_width]
    // Double the size in a [0,1] coordinates system, but opengl is [-1,1].
    vertex.position *= vec4(dimension * 2, 1, 1);

    // Scale everything back down from pixel coordinates to [-1,1] domain.
    vertex.position *= proj_vec;

    // Move everything by the coordinates of the element.
    vertex.position += vec4(coord * 2, 0, 0) * proj_vec;

    // Recenter the whole viewport.
    vertex.position += vec4(-1, -1, 0, 0);

    vertex.tex_coord = tex_coord;
    gl_Position = vertex.position;
}

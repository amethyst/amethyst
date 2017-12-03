// TODO: Needs documentation.

#version 150 core

// std140 is a cross platform layout.
layout (std140) uniform VertexArgs {
    uniform vec4 proj_vec;
    uniform vec2 coord;
    uniform vec2 dimension;
};

in vec3 position;
in vec2 tex_coord;

out VertexData {
  vec4 position;
  vec2 tex_coord;
} vertex;

void main() {
    vertex.position = vec4(position, 1);
    vertex.position *= vec4(dimension, 1, 1);
    vertex.position += vec4(coord, 0, 0);
    vertex.position *= proj_vec;
    vertex.position += vec4(-1, 1, 0, 0);
    vertex.tex_coord = tex_coord;
    gl_Position = vertex.position;
}

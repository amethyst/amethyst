// TODO: Needs documentation.

#version 150 core

layout (std140) uniform VertexArgs {
    uniform vec2 coord;
    uniform vec2 dimension;
    uniform vec2 screen_dimensions;
};

in vec3 position;
in vec2 tex_coord;

out VertexData {
  vec4 position;
  vec2 tex_coord;
} vertex;

void main() {
    vertex.position = vec4(position, 1.0);
    vertex.tex_coord = tex_coord;
    gl_Position = vertex.position;
}

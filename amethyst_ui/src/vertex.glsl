// TODO: Needs documentation.

#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
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
    vertex.position = vec4(position * vec3(dimension / screen_dimensions, 1.0) + vec3(coord / screen_dimensions, 0.0), -1.0) * 2.0 - 1.0;
    vertex.tex_coord = tex_coord;
    gl_Position = proj * vertex.position;
}

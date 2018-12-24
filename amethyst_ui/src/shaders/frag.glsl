// TODO: Needs documentation.

#version 150 core

uniform sampler2D albedo;

in VertexData {
  vec4 position;
  vec2 tex_coord;
  vec4 color;
} vertex;

out vec4 color;

void main() {
    color = texture(albedo, vertex.tex_coord) * vertex.color;
}

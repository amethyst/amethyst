// TODO: Needs documentation.

#version 150 core

uniform sampler2D albedo;

in VertexData {
  vec4 position;
  vec2 tex_coord;
} vertex;

out vec4 color;

void main() {
    color = vec4(1.0, 0.0, 0.0, 1.0);
    //color = texture(albedo, vertex.tex_coord);
}

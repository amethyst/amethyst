// TODO: Needs documentation.

#version 150 core

in VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec4 color;
} vertex;

out vec4 color;

void main() {
    color = vertex.color;
}

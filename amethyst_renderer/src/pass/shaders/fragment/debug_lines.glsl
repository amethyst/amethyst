// TODO: Needs documentation.

#version 150 core

in VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

out vec4 out_color;

void main() {
    // out_color = vec4(1.0, 1.0, 1.0, 1.0);
    out_color = vertex.color;
}

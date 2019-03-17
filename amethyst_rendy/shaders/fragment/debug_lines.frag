#version 450

layout(location = 0) in VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = vertex.color;
}

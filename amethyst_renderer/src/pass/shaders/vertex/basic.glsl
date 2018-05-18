// TODO: Needs documentation.

#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model;
};

in vec3 position;
in vec3 normal;
in vec3 tangent;
in vec2 tex_coord;

out VertexData {
    vec4 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
} vertex;

void main() {
    vertex.position = model * vec4(position, 1.0);
    vertex.normal = mat3(model) * normal;
    vertex.tangent = mat3(model) * tangent;
    vertex.tex_coord = tex_coord;
    gl_Position = proj * view * vertex.position;
}

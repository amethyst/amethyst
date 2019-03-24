#version 450

layout(std140, set = 0, binding = 0) uniform VertexArgs {
    mat4 proj;
    mat4 view;
    mat4 model;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 tangent;
layout(location = 3) in vec2 tex_coord;

layout(location = 0) out VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
} vertex;

void main() {
    vec4 vertex_position = model * vec4(position, 1.0);
    vertex.position = vertex_position.xyz;
    vertex.normal = mat3(model) * normal;
    vertex.tangent = mat3(model) * tangent;
    vertex.tex_coord = tex_coord;
    gl_Position = proj * view * vertex_position;
}

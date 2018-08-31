// TODO: Needs documentation.

#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model;
};

in vec3 position;
in vec3 normal;

out VertexData {
    vec3 position;
    vec3 normal;
} vertex;

void main() {
    vec4 vertex_position = model * vec4(position, 1.0);
    vertex.position = vertex_position.xyz;
    gl_Position = proj * view * vertex_position;

    mat3 correctionMatrix = mat3(transpose(inverse(view * model)));
    vertex.normal = normalize(vec3(proj * vec4(correctionMatrix * normal, 0.0)));
}

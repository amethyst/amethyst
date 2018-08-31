// TODO: Needs documentation.

#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model; /* Not actually used, all our lines are in world coordinates */
};

in vec3 position;
in vec4 color;
in vec3 normal;

out VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

void main() {
    vertex.position = position;
    gl_Position = proj * view * vec4(position, 1.0);

    mat3 correctionMatrix = mat3(transpose(inverse(view * model)));
    vertex.normal = normalize(vec3(proj * vec4(correctionMatrix * vec3(normal), 0.0)));

    vertex.color = color;
}

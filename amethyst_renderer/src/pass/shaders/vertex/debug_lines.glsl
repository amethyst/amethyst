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
    // vertex.normal = vec3(proj * view * vec4(normal, 0.0));
    vertex.normal = normal;
    vertex.color = color;
}

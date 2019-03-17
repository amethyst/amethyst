// TODO: Needs documentation.

#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model; /* Not actually used, lines are in world coordinates */
};

in vec3 position;
in vec4 color;
in vec3 normal;

uniform vec3 camera_position;

out VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

void main() {
    vertex.position = position;
    vertex.normal = normal;
    vertex.color = color;
}

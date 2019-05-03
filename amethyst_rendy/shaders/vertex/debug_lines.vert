#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec3 normal;


layout(std140, set = 1, binding = 0) uniform _ {
    vec3 camera_position;
};

layout(location = 2) out VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

void main() {
    vertex.position = position;
    vertex.normal = normal;
    vertex.color = color;
}

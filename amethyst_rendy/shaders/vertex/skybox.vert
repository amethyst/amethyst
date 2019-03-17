#version 450

layout(std140, set = 0, binding = 0) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;

layout(location = 0) out VertexData {
    vec3 position;
    vec2 tex_coord;
} vertex;

void main() {
    mat4 view_without_translation = view;
    view_without_translation[3].xyz = vec3(0.0f, 0.0f, 0.0f);
    vec4 vertex_position = model * vec4(position, 1.0);

    vertex.position = vertex_position.xyz;
    vertex.tex_coord = tex_coord;

    gl_Position = (proj * view_without_translation * vertex_position).xyww;
}

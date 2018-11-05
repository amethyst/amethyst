// TODO: Needs documentation.

#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model;
};

in vec3 position;
in vec3 normal;
in vec2 tex_coord;

out VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
} vertex;

void main() {
    mat4 view_without_translation = view;
    view_without_translation[3].xyz = vec3(0.0f, 0.0f, 0.0f);
    vec4 vertex_position = model * vec4(position, 1.0);

    vertex.position = vertex_position.xyz;
    vertex.normal = mat3(model) * normal;
    vertex.tex_coord = tex_coord;

    gl_Position = (proj * view_without_translation * vertex_position).xyww;
}

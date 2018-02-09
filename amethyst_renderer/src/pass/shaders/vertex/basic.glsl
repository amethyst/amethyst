#version 300 es

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model;
};

in vec3 position;
in vec3 normal;
in vec3 tangent;
in vec2 tex_coord;

out vec4 vertex_position;
out vec3 vertex_normal;
out vec3 vertex_tangent;
out vec2 vertex_tex_coord;

void main() {
    vertex_position = model * vec4(vertex_position.xyz, 1.0);
    vertex_normal = mat3(model) * vertex_normal;
    vertex_tangent = mat3(model) * vertex_tangent;
    vertex_tex_coord = vertex_tex_coord;
    gl_Position = proj * view * vertex_position;
}

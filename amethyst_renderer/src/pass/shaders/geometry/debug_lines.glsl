// TODO: Needs documentation.

#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model; /* Not actually used, all our lines are in world coordinates */
};

in VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex_in[1];

out VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

uniform vec3 camera_position;
uniform float line_width;

void EmitLine (int id) {
    vec3 cam_dir = normalize(vertex_in[id].position - camera_position);
    vec3 right = normalize(cross(cam_dir, vertex_in[id].normal));

    vec3 width_vector = right * line_width * 0.5;

    vertex.color = vertex_in[id].color;
    vec3 pos = vertex_in[id].position - width_vector;
    gl_Position = proj * view * vec4(pos, 1.0);
    EmitVertex();

    vertex.color = vertex_in[id].color;
    pos = vertex_in[id].position + width_vector;
    gl_Position = proj * view * vec4(pos, 1.0);
    EmitVertex();

    vertex.color = vertex_in[id].color;
    pos = vertex_in[id].position + vertex_in[id].normal - width_vector;
    gl_Position = proj * view * vec4(pos, 1.0);
    EmitVertex();

    vertex.color = vertex_in[id].color;
    pos = vertex_in[id].position + vertex_in[id].normal + width_vector;
    gl_Position = proj * view * vec4(pos, 1.0);
    EmitVertex();
    EndPrimitive();
}

void main() {
    EmitLine(0);
}

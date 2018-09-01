#version 150 core

in VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex_in[3];

out VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

layout (triangles) in;
layout (line_strip, max_vertices = 6) out;

const float MAGNITUDE = 0.1;

void EmitLine (int id) {
    vertex.color = vertex_in[id].color;
    gl_Position = gl_in[id].gl_Position;
    EmitVertex();
    vertex.color = vertex_in[id].color;
    gl_Position = gl_in[id].gl_Position + vec4(vertex_in[id].normal, 0);
    EmitVertex();
    EndPrimitive();
}

void main()
{
    EmitLine(0);
    EmitLine(1);
    EmitLine(2);
}
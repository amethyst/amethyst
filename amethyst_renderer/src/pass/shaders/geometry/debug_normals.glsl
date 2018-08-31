#version 150 core

in VertexData {
    vec3 position;
    vec3 normal;
} vertex[3];

layout (triangles) in;
layout (line_strip, max_vertices = 6) out;

const float MAGNITUDE = 0.1;

void main()
{
    gl_Position = gl_in[0].gl_Position; 
    EmitVertex();
    gl_Position = gl_in[0].gl_Position + vec4(vertex[0].normal, 0) * MAGNITUDE; 
    EmitVertex();
    EndPrimitive();

    gl_Position = gl_in[1].gl_Position; 
    EmitVertex();
    gl_Position = gl_in[1].gl_Position + vec4(vertex[1].normal, 0) * MAGNITUDE;
    EmitVertex();
    EndPrimitive();

    gl_Position = gl_in[2].gl_Position; 
    EmitVertex();
    gl_Position = gl_in[2].gl_Position + vec4(vertex[2].normal, 0) * MAGNITUDE;
    EmitVertex();
    EndPrimitive();
}
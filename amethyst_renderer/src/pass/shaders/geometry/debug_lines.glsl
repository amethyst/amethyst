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
} vertex_in[3];

out VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

layout (triangles) in;
layout (triangle_strip, max_vertices = 12) out;

const float WIDTH = 2.0 / 500.0;
const float HALF_WIDTH = WIDTH / 2.0;

void EmitLine (int id) {
    vec3 width_vector = vec3(HALF_WIDTH, 0, 0);

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

void main()
{
    EmitLine(0);
    EmitLine(1);
    EmitLine(2);
}
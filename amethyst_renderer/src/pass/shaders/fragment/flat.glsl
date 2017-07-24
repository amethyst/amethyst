#version 150 core

uniform sampler2D albedo;

in VertexData {
    vec4 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
} vertex;

out vec4 color;

void main() {
    color = texture(albedo, vertex.tex_coord);
}

// TODO: Needs documentation.

#version 150 core

uniform sampler2D albedo;

layout (std140) uniform AlbedoOffset {
    vec2 u_offset;
    vec2 v_offset;
} albedo_offset;

in VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
} vertex;

out vec4 color;

float tex_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

vec2 tex_coords(vec2 coord, vec2 u, vec2 v) {
    return vec2(tex_coord(coord.x, u), tex_coord(coord.y, v));
}

void main() {
    color = texture(albedo, tex_coords(vertex.tex_coord, albedo_offset.u_offset, albedo_offset.v_offset));
}

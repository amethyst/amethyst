// TODO: Needs documentation.

#version 150 core

uniform sampler2D albedo;

layout (std140) uniform AlbedoOffset {
    vec2 u_offset;
    vec2 v_offset;
} albedo_offset;

in vec2 tex_uv;

out vec4 color;

// coord = 0.0 to 1.0 texture coordinate
float texture_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

// coords = 0.0 to 1.0 texture coordinates
vec2 texture_coords(vec2 coords, vec2 u, vec2 v) {
    return vec2(texture_coord(coords.x, u), texture_coord(coords.y, v));
}

void main() {
    color = texture(albedo, texture_coords(tex_uv, albedo_offset.u_offset, albedo_offset.v_offset));
}

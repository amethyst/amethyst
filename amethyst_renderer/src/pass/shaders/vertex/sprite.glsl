#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model;
};

layout (std140) uniform SpriteArgs {
    // Sprite width and height.
    uniform vec2 sprite_dimensions;
    // Pixel offsets for the sprite. Used to shift the sprite left and down.
    uniform vec2 offsets;
};

layout (std140) uniform AlbedoOffset {
    vec2 u_offset;
    vec2 v_offset;
} albedo_offset;

out vec2 tex_uv;
out vec2 uv;

// Position coordinates for two triangles that form a quad.
const vec2 positions[6] = vec2[](
    // First triangle
    vec2(0.0, 0.0), // Left bottom
    vec2(1.0, 0.0), // Right bottom
    vec2(1.0, 1.0), // Right top

    // Second triangle
    vec2(1.0, 1.0), // Right top
    vec2(0.0, 1.0), // Left top
    vec2(0.0, 0.0)  // Left bottom
);

// coord = 0.0 to 1.0 texture coordinate
float texture_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

// coords = 0.0 to 1.0 texture coordinates
vec2 texture_coords(vec2 coords, vec2 u, vec2 v) {
    return vec2(texture_coord(coords.x, u), texture_coord(coords.y, v));
}

void main() {
    vec2 position = positions[gl_VertexID];

    uv = position * sprite_dimensions;
    tex_uv = texture_coords(position, albedo_offset.u_offset, albedo_offset.v_offset);

    vec4 vertex = vec4(uv, 0.0, 1.0);
    gl_Position = proj * view * model * vertex;
}

#version 150 core

layout (std140) uniform ViewArgs {
    mat4 proj;
    mat4 view;
};

//
in vec2 half_diag;
// Pixel offsets for the sprite. Used to shift the sprite left and down.
in vec2 offsets;

in vec2 u_offset;
in vec2 v_offset;

// Whether to flip the sprite horizontally
in bool flip_horizontal;
// Whether to flip the sprite vertically
in bool flip_vertical;

out vec2 uv;
out vec2 tex_uv;

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

const vec2 positions_flip[6] = vec2[](
    // First triangle
    vec2(1.0, 1.0), // Left bottom
    vec2(0.0, 1.0), // Right bottom
    vec2(0.0, 0.0), // Right top

    // Second triangle
    vec2(0.0, 0.0), // Right top
    vec2(1.0, 0.0), // Left top
    vec2(1.0, 1.0)  // Left bottom
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
    // The vertex position needs to be adjusted by the offset.
    //
    // In the following diagram, the sprite (delimited by the `[]` brackets) has width=6 and
    // offsets[0]: 2.
    //
    // ```
    // P: the pixel where the sprite starts to be drawn (E - offsets[0]).
    // Q: the first pixel where the sprite is out of bounds (E - offsets[0] + sprite_w + 1).
    // E: the position of the entity, which is the center line where the sprite should be flipped.
    //
    // |------------|
    // |   [ E >]   |    P = E - offsets[0]
    // |------------|      = 5 - 2
    //     P ^   Q         = 3
    //  0123456789AB
    //                   Q = E - offsets[0] + sprite_w + 1
    //                     = 5 - 2 + 6 + 1
    //                     = 9
    // ```
    //
    // When flipped horizontally, the entity's center should remain on E, and the vertices mirrored
    // around it.
    //
    // ```
    // |------------|
    // |  [< E ]    |    P = E + offsets[0]
    // |------------|      = 5 + 2
    //   Q   ^ P           = 7
    //  0123456789AB
    //                   Q = E + offsets[0] - sprite_w
    //                     = 5 + 2 - 6
    //                     = 1
    // ```

    float u;
    float v;
    float tex_u;
    float tex_v;

    if (flip_horizontal) {
        tex_u = positions_flip[gl_VertexID][0];
        u = positions[gl_VertexID][0] * half_diag[0] + offsets[0] - half_diag[0];
    } else {
        tex_u = positions[gl_VertexID][0];
        u = tex_u * half_diag[0] + offsets[0] - half_diag[0];
    }

    if (flip_vertical) {
        tex_v = positions_flip[gl_VertexID][1];
        v = positions[gl_VertexID][1] * half_diag[1] + offsets[1] - half_diag[1];
    } else {
        tex_v = positions[gl_VertexID][1];
        v = tex_v * half_diag[1] + offsets[1] - half_diag[1];
    }

    uv = vec2(u, v);
    tex_uv = texture_coords(vec2(tex_u, tex_v), u_offset, v_offset);

    vec4 vertex = vec4(uv, 0.0, 1.0);
    gl_Position = proj * view * vertex;
}

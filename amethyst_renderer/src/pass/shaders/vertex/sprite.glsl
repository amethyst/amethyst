#version 150 core

layout (std140) uniform ViewArgs {
    mat4 proj;
    mat4 view;
};

// Quad transform.
in vec2 size;
in vec2 offset;

// Texture quad.
in vec2 u_offset;
in vec2 v_offset;

out vec2 uv;
out vec2 tex_uv;

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

// coords = 0.0 to 1.0 texture coordinates
vec2 texture_coords(vec2 coords, vec2 u, vec2 v) {
    return vec2(mix(u.x, u.y, coords.x), mix(v.x, v.y, coords.y));
}

void main() {
    float tex_u = positions[gl_VertexID][0];
    float tex_v = positions[gl_VertexID][1];

    float u = (tex_u - 1.0f) * size[0] + offset[0];
    float v = (tex_v - 1.0f) * size[1] + offset[1];

    tex_uv = texture_coords(vec2(tex_u, tex_v), u_offset, v_offset);

    vec4 vertex = vec4(u, v, 0.0, 1.0);
    gl_Position = proj * view * vertex;
}

#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 proj_view;
};

// Quad transform.
layout(location = 0) in vec2 dir_x;
layout(location = 1) in vec2 dir_y;
layout(location = 2) in vec2 pos;
layout(location = 3) in vec2 u_offset;
layout(location = 4) in vec2 v_offset;
layout(location = 5) in float depth;
layout(location = 6) in vec4 color;

layout(location = 0) out VertexData {
    vec2 tex_uv;
    vec4 color;
} vertex;

const vec2 positions[4] = vec2[](
    vec2(0.5, -0.5), // Right bottom
    vec2(-0.5, -0.5), // Left bottom
    vec2(0.5, 0.5), // Right top
    vec2(-0.5, 0.5) // Left top
);

// coords = 0.0 to 1.0 texture coordinates
vec2 texture_coords(vec2 coords, vec2 u, vec2 v) {
    return vec2(mix(u.x, u.y, coords.x+0.5), mix(v.x, v.y, coords.y+0.5));
}

void main() {
    float tex_u = positions[gl_VertexIndex][0];
    float tex_v = positions[gl_VertexIndex][1];

    vertex.tex_uv = texture_coords(vec2(tex_u, tex_v), u_offset, v_offset);
    vertex.color = color;
    vec2 final_pos = pos + tex_u * dir_x + tex_v * dir_y;
    vec4 vertex = vec4(final_pos, depth, 1.0);
    gl_Position = proj_view * vertex;
}

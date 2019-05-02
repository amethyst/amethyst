#version 450

struct UvOffset {
    vec2 u_offset;
    vec2 v_offset;
};

layout(std140, set = 1, binding = 0) uniform Material {
    UvOffset uv_offset;
    float alpha_cutoff;
};

layout(set = 1, binding = 1) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec3 position;
    vec2 tex_coord;
    vec4 color;
} vertex;

layout(location = 0) out vec4 out_color;

float tex_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

vec2 tex_coords(vec2 coord, vec2 u, vec2 v) {
    return vec2(tex_coord(coord.x, u), tex_coord(coord.y, v));
}

void main() {
    vec4 albedo = texture(albedo, tex_coords(vertex.tex_coord, uv_offset.u_offset, uv_offset.v_offset));
    if(albedo.w < alpha_cutoff) discard;
    out_color = albedo * vertex.color;
}

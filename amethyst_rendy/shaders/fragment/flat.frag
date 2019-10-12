#version 450

#include "header/math.frag"

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

void main() {
    vec4 albedo = texture(albedo, tex_coords(vertex.tex_coord, uv_offset));
    if(albedo.w < alpha_cutoff) discard;
    out_color = albedo * vertex.color;
}

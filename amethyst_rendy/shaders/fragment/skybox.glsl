// TODO: Needs documentation.

#version 150 core

in VertexData {
    vec3 position;
    vec2 tex_coord;
} vertex;

out vec4 out_color;

uniform vec3 zenith_color;
uniform vec3 nadir_color;

void main() {
    vec3 normalized_position = normalize(vertex.position.xyz);
    vec3 horizon_color = mix(nadir_color, zenith_color, smoothstep(-1., 1., normalized_position.y));
    out_color = vec4(horizon_color, 1.0f);
}

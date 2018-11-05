// TODO: Needs documentation.

#version 150 core

in VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
} vertex;

out vec4 out_color;

vec3 zenith_color = vec3(0.75, 1.0, 1.0);
vec3 nadir_color = vec3(0.2, 0.4, 0.45);

void main() {
    vec3 horizon_color = mix(nadir_color, zenith_color, smoothstep(-1., 1., vertex.position.y));
    out_color = vec4(horizon_color, 1.0f);
}

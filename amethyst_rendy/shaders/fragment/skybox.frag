#version 450

layout(early_fragment_tests) in;

layout(location = 0) in VertexData {
    vec3 position;
    vec2 tex_coord;
} vertex;

layout(location = 0) out vec4 out_color;

layout(std140, set = 1, binding = 0) uniform _ {
    vec3 zenith_color;
    vec3 nadir_color;
};

void main() {
    vec3 normalized_position = normalize(vertex.position.xyz);
    vec3 horizon_color = mix(nadir_color, zenith_color, smoothstep(-1., 1., normalized_position.y));
    out_color = vec4(horizon_color, 1.0f);
}

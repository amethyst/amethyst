#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
};

layout(std140, set = 1, binding = 0) uniform DebugLinesArgs {
    uniform vec2 screen_space_thickness;
};

layout(location = 0) in vec3 position_a;
layout(location = 1) in vec4 color_a;
layout(location = 2) in vec3 position_b;
layout(location = 3) in vec4 color_b;

const mat2 dir_mats[2] = mat2[](
    mat2(0.0, 1.0, -1.0, 0.0),
    mat2(0.0, -1.0, 1.0, 0.0)
);

layout(location = 0) out VertexData {
    vec4 color;
} vertex;

void main() {

    mat4 proj_view = proj * view;
    vec4 projected_a = proj_view * vec4(position_a, 1.0);
    vec4 projected_b = proj_view * vec4(position_b, 1.0);
    vec2 screen_a = projected_a.xy / projected_a.w;
    vec2 screen_b = projected_b.xy / projected_b.w;

    vec2 normal = normalize((screen_b - screen_a)) * dir_mats[gl_VertexIndex & 1];
    float factor = float((gl_VertexIndex & 2) >> 1);

    normal *= mix(projected_a.w, projected_b.w, factor) * 1.0;
    normal.x /= screen_space_thickness.x;
    normal.y /= screen_space_thickness.y;

    vertex.color = mix(color_a, color_b, factor);
    gl_Position = mix(projected_a, projected_b, factor) + vec4(normal, 0.0, 0.0);
}

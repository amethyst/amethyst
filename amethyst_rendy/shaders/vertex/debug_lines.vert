#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 proj_view;
};

layout(std140, set = 1, binding = 0) uniform DebugLinesArgs {
    uniform vec2 screen_space_thickness;
};

layout(location = 0) in vec3 position_a;
layout(location = 1) in vec4 color_a;
layout(location = 2) in vec3 position_b;
layout(location = 3) in vec4 color_b;

layout(location = 0) out VertexData {
    vec4 color;
} vertex;

void main() {
    float factor = float(gl_VertexIndex >> 1);
    vertex.color = mix(color_a, color_b, factor);

    vec4 projected_a = proj_view * vec4(position_a, 1.0);
    vec4 projected_b = proj_view * vec4(position_b, 1.0);
    vec4 proj_current = mix(projected_a, projected_b, factor);

    if (proj_current.w < 0) {
        // vertex behind camera clip plane
        vec4 proj_next = mix(projected_b, projected_a, factor);
        vec3 clip_space_dir =  normalize(proj_current.xyw - proj_next.xyw);
        float coef = -proj_current.w / clip_space_dir.z;
        vec3 intersect_pos = proj_current.xyw + (clip_space_dir * coef);
        gl_Position = vec4(intersect_pos.x, intersect_pos.y, 0, intersect_pos.z);
    } else {
        vec2 screen_a = projected_a.xy / projected_a.w;
        vec2 screen_b = projected_b.xy / projected_b.w;
        vec2 dir = normalize(screen_b - screen_a);

        vec2 normal;
        if (mod(gl_VertexIndex, 2) == 0) {
            normal = vec2(-dir.y, dir.x);
        } else {
            normal = vec2(dir.y, -dir.x);
        }
        
        normal *= proj_current.w * screen_space_thickness;
        gl_Position = proj_current + vec4(normal, 0.0, 0.0);
    }
}

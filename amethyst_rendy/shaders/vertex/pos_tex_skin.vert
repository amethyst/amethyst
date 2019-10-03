#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 proj_view;
};

layout(std430, set = 2, binding = 0) readonly buffer JointTransforms {
    mat4 joints[];
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in uvec4 joint_ids;
layout(location = 3) in vec4 joint_weights;
layout(location = 4) in mat4 model; // instance rate
layout(location = 8) in vec4 tint; // instance rate
layout(location = 9) in uint joints_offset; // instance rate

layout(location = 0) out VertexData {
    vec3 position;
    vec2 tex_coord;
    vec4 color;
} vertex;

void main() {
    mat4 joint_transform =
        joint_weights.x * joints[int(joints_offset + joint_ids.x)] +
        joint_weights.y * joints[int(joints_offset + joint_ids.y)] +
        joint_weights.z * joints[int(joints_offset + joint_ids.z)] +
        joint_weights.w * joints[int(joints_offset + joint_ids.w)];

    vec4 vertex_position = model * joint_transform * vec4(position, 1.0);
    vertex.position = vertex_position.xyz;
    vertex.tex_coord = tex_coord;
    vertex.color = tint;
    gl_Position = proj_view * vertex_position;
}

// TODO: Needs documentation.

#version 150 core

layout (std140) uniform JointTransforms {
    mat4 joints[100];
};

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model;
};

in vec3 position;
in vec3 normal;
in vec3 tangent;
in vec2 tex_coord;
in uvec4 joint_ids;
in vec4 joint_weights;

out VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
} vertex;



void main() {
    mat4 joint_transform = joint_weights.x * joints[int(joint_ids.x)] +
        joint_weights.y * joints[int(joint_ids.y)] +
        joint_weights.z * joints[int(joint_ids.z)] +
        joint_weights.w * joints[int(joint_ids.w)];

    vec4 vertex_position = model * joint_transform * vec4(position, 1.0);
    mat3 mat3_transform = mat3(model) * mat3(joint_transform);
    vertex.position = vertex_position.xyz;
    vertex.normal = mat3_transform * normal;
    vertex.tangent = mat3_transform * tangent;
    vertex.tex_coord = tex_coord;
    gl_Position = proj * view * vertex_position;
}

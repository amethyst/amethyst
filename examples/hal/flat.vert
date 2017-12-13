#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 0) out vec4 out_color;

layout(binding=0, set=0) uniform TrProjView {
    mat4 transform;
    // mat4 view;
    // mat4 projection;
};


void main() {
    out_color = color;
    gl_Position = vec4(position, 1.0) * transform;
}

// TODO: Needs documentation.

#version 150 core

uniform sampler2D albedo;

in vec2 tex_uv;

out vec4 color;

void main() {
    color = texture(albedo, tex_uv);
}

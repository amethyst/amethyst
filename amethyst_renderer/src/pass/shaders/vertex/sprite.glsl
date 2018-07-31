#version 150 core

layout (std140) uniform VertexArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 model;
};

layout (std140) uniform SpriteArgs {
    // Sprite width and height.
    uniform vec2 sprite_dimensions;
    // Pixel offsets for the sprite. Used to shift the sprite left and down.
    uniform vec2 offsets;
};

out vec2 tex_uv;
out vec2 uv;

// Position coordinates for two triangles that form a quad.
const vec2 positions[6] = vec2[](
    // First triangle
    vec2(0.0, 0.0), // Left bottom
    vec2(1.0, 0.0), // Right bottom
    vec2(1.0, 1.0), // Right top

    // Second triangle
    vec2(1.0, 1.0), // Right top
    vec2(0.0, 1.0), // Left top
    vec2(0.0, 0.0)  // Left bottom
);

void main() {
    tex_uv = positions[gl_VertexID];
    uv = tex_uv * sprite_dimensions;

    vec4 vertex = vec4(uv, 0.0, 1.0);
    gl_Position = proj * view * model * vertex;
}

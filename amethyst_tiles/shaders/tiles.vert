#version 450

layout(std140, set = 0, binding = 0) uniform TileMapArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 map_coordinate_transform;
    uniform mat4 map_transform;
    // We assume coordinates are uniform for tiles, so we can actually store the sprite information here
    uniform vec2 sprite_dimensions;
};

// Quad transform.
layout(location = 0) in vec2 u_offset;
layout(location = 1) in vec2 v_offset;
layout(location = 2) in vec4 color;
layout(location = 3) in uvec3 tile_coordinate;

layout(location = 0) out VertexData {
    vec2 tex_uv;
    vec4 color;
} vertex;

const vec2 positions[4] = vec2[](
    vec2(0.5, -0.5), // Right bottom
    vec2(-0.5, -0.5), // Left bottom
    vec2(0.5, 0.5), // Right top
    vec2(-0.5, 0.5) // Left top
);

// coords = 0.0 to 1.0 texture coordinates
vec2 texture_coords(vec2 coords, vec2 u, vec2 v) {
    return vec2(mix(u.x, u.y, coords.x+0.5), mix(v.x, v.y, coords.y+0.5));
}

void main() {
    float tex_u = positions[gl_VertexIndex][0];
    float tex_v = positions[gl_VertexIndex][1];


    vec2 ddir_x = (map_transform[0] * sprite_dimensions.x).xy;
    vec2 ddir_y = (map_transform[1] * -sprite_dimensions.y).xy;

    vec4 coord = vec4(float(tile_coordinate.x), -float(tile_coordinate.y), float(tile_coordinate.z), 1.0);

    vec4 world_coordinate = map_coordinate_transform * coord;
    world_coordinate = world_coordinate * transpose(map_transform);

    vertex.tex_uv = texture_coords(vec2(tex_u, tex_v), u_offset, v_offset);
    vertex.color = color;

    vec2 final_pos = world_coordinate.xy + tex_u * ddir_x + tex_v * ddir_y;
    vec4 vertex = vec4(final_pos, world_coordinate.z, 1.0);
    gl_Position = proj * view * vertex;
}

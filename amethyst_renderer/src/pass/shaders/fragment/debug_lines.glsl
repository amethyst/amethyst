// TODO: Needs documentation.

#version 150 core

in VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

out vec4 out_color;

mat4 THRESHOULD_MATRIX = mat4(
     1.0 / 17.0,  9.0 / 17.0,  3.0 / 17.0, 11.0 / 17.0,
    13.0 / 17.0,  5.0 / 17.0, 15.0 / 17.0,  7.0 / 17.0,
     4.0 / 17.0, 12.0 / 17.0,  2.0 / 17.0, 10.0 / 17.0,
    16.0 / 17.0,  8.0 / 17.0, 14.0 / 17.0,  6.0 / 17.0
);

float checker(vec2 uv, float tiling) {
    float x = floor(tiling * uv.x);
    float y = floor(tiling * uv.y); 
    float result = mod(x + y, 2.0);
    return sign(result);
}

void main() {
    vec4 color = vertex.color;

    // if (color.a - THRESHOULD_MATRIX[int(mod(gl_FragCoord.x, 4))][int(mod(gl_FragCoord.y, 4))] < 0) discard;

    out_color = vec4(vertex.color.rgba);
}

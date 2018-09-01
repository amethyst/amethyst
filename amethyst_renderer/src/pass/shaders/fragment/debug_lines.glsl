// TODO: Needs documentation.

#version 150 core

in VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

out vec4 out_color;

float checker(vec2 uv, float tiling) {
    float x = floor(tiling * uv.x);
    float y = floor(tiling * uv.y); 
    float result = mod(x + y, 2.0);
    return sign(result);
}

void main() {
    vec4 color = vertex.color;

    // if (checker(gl_FragCoord.xy, 1.0) > 0) discard;

    out_color = vec4(vertex.color.rgb, 1.0);
}

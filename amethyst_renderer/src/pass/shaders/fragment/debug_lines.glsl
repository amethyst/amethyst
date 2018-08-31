// TODO: Needs documentation.

#version 150 core

in VertexData {
    vec3 position;
    vec4 color;
    vec3 normal;
} vertex;

out vec4 out_color;

float checker(vec2 uv, float repeats) {
    float cx = floor(repeats * uv.x);
    float cy = floor(repeats * uv.y); 
    float result = mod(cx + cy, 2.0);
    return sign(result);
}

void main() {
    vec4 color = vertex.color;

    // if (checker(gl_FragCoord.xy, 0.9) > 0) 
    //     discard;

    out_color = vertex.color;
}

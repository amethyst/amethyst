// TODO: Needs documentation.

#version 150 core

uniform vec3 camera_position;

in VertexData {
    vec3 position;
} vertex;

out vec4 out_color;

float checker(vec2 uv, float repeats) {
    float cx = floor(repeats * uv.x);
    float cy = floor(repeats * uv.y); 
    float result = mod(cx + cy, 2.0);
    return sign(result);
}

void main() {
    vec4 color = vec4(1.0, 1.0, 1.0, 1.0);

    // if (checker(gl_FragCoord.xy, 0.5) > 0) 
    //     discard;

    out_color = color;
}

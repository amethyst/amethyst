#version 450

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec2 tex_uv;
    vec4 color;
    float time;
} vertex;
layout(location = 0) out vec4 out_color;

void main() {

    float noise = pow(texture(albedo, vertex.tex_uv + vec2(0,vertex.time/2.0)).r ,0.45454545);
    float gradiant = vertex.tex_uv.y;
    float S1 = step(noise,gradiant + 0.25);
    float S2 = step(noise,gradiant);
    float S3 = step(noise,gradiant - 0.25);

    float L1 = S1 - S2;
    float L2 = S2 - S3;



    vec4 color = mix( vec4(1.0,1.0,0.0,1.0),  vec4(1.0,0.0,0.0,1.0), L1);
    color = mix( color,  vec4(1.0,0.5,0.0,1.0), L2);

    color.a = S1;
    if (color.a == 0.0) {
        discard;
    }
    out_color = color;
}

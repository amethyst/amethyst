#ifndef MATH_FRAG
#define MATH_FRAG

const float PI = 3.14159265359;

struct UvOffset {
    vec2 u_offset;
    vec2 v_offset;
};

float tex_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

vec2 tex_coords(vec2 coord, UvOffset offset) {
    return vec2(tex_coord(coord.x, offset.u_offset), tex_coord(coord.y, offset.v_offset));
}
 
vec3 schlick_fresnel(float HdotV, vec3 fresnel_base) {
    return fresnel_base + (1.0 - fresnel_base) * pow(1.0 - HdotV, 5.0);
}

float ggx_normal_distribution(vec3 N, vec3 H, float a) {
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return (a2 + 0.0000001) / denom;
}

float ggx_geometry(float NdotV, float NdotL, float r2) {
    float a1 = r2 + 1.0;
    float k = a1 * a1 / 8.0;
    float denom = NdotV * (1.0 - k) + k;
    float ggx1 = NdotV / denom;
    denom = NdotL * (1.0 - k) + k;
    float ggx2 = NdotL / denom;
    return ggx1 * ggx2;
}

float s_curve (float x) {
		x = x * 2.0 - 1.0;
		return -x * abs(x) * 0.5 + x + 0.5;
}

#endif

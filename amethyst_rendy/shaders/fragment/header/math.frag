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

float saturate(float s) {
    return clamp(s, 0.0, 1.0);
}
vec2 saturate2(vec2 s) {
    return vec2(clamp(s.x, 0.0, 1.0), clamp(s.y, 0.0, 1.0));
}
vec3 saturate3(vec3 s) {
    return vec3(clamp(s.x, 0.0, 1.0), clamp(s.y, 0.0, 1.0), clamp(s.z, 0.0, 1.0));
}
vec4 saturate4(vec4 s) {
    return vec4(clamp(s.x, 0.0, 1.0), clamp(s.y, 0.0, 1.0), clamp(s.z, 0.0, 1.0), clamp(s.w, 0.0, 1.0));
}

float sqr(float x) { return x*x; }


mat3 rotationMatrix3(vec3 axis, float cosTheta) {
    axis = normalize(axis);
    float c = cosTheta;
    float s = sqrt(1.0-c*c);
    float oc = 1.0 - c;
    
    return mat3(oc * axis.x * axis.x + c,           oc * axis.x * axis.y - axis.z * s,  oc * axis.z * axis.x + axis.y * s,  
                oc * axis.x * axis.y + axis.z * s,  oc * axis.y * axis.y + c,           oc * axis.y * axis.z - axis.x * s,  
                oc * axis.z * axis.x - axis.y * s,  oc * axis.y * axis.z + axis.x * s,  oc * axis.z * axis.z + c           );
}

mat4 rotationMatrix4(vec3 axis, float cosTheta) {
    axis = normalize(axis);
    float c = cosTheta;
    float s = sqrt(1.0-c*c);
    float oc = 1.0 - c;
    
    return mat4(oc * axis.x * axis.x + c,           oc * axis.x * axis.y - axis.z * s,  oc * axis.z * axis.x + axis.y * s,  0.0,
                oc * axis.x * axis.y + axis.z * s,  oc * axis.y * axis.y + c,           oc * axis.y * axis.z - axis.x * s,  0.0,
                oc * axis.z * axis.x - axis.y * s,  oc * axis.y * axis.z + axis.x * s,  oc * axis.z * axis.z + c,           0.0,
                0.0, 0.0, 0.0, 1.0);
}

#endif

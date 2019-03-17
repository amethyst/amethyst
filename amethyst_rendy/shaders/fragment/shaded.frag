#version 450

struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
};

struct DirectionalLight {
    vec3 color;
    vec3 direction;
};

layout(set = 2, binding = 0) uniform Environment {
    vec3 ambient_color;
    vec3 camera_position; 
    int point_light_count;
    int directional_light_count;
};

layout(set = 2, binding = 1) uniform PointLights {
    PointLight plight[128];
};

layout(set = 2, binding = 2) uniform DirectionalLights {
    DirectionalLight dlight[16];
};

struct UvOffset {
    vec2 u_offset;
    vec2 v_offset;
};

layout(set = 1, binding = 1) uniform sampler2D albedo;
layout(set = 1, binding = 2) uniform sampler2D emission;

layout(set = 1, binding = 0) uniform Material {
    UvOffset albedo_offset;
    UvOffset emission_offset;
};

layout(location = 0) in VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
} vertex;

layout(location = 0) out vec4 out_color;

float tex_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

vec2 tex_coords(vec2 coord, vec2 u, vec2 v) {
    return vec2(tex_coord(coord.x, u), tex_coord(coord.y, v));
}

void main() {
    vec4 color = texture(albedo, tex_coords(vertex.tex_coord, albedo_offset.u_offset, albedo_offset.v_offset));
    vec4 ecolor = texture(emission, tex_coords(vertex.tex_coord, emission_offset.u_offset, emission_offset.v_offset));
    vec3 lighting = vec3(0.0);
    vec3 normal = normalize(vertex.normal);
    for (uint i = 0u; i < point_light_count; i++) {
        // Calculate diffuse light
        vec3 light_dir = normalize(plight[i].position - vertex.position);
        float diff = max(dot(light_dir, normal), 0.0);
        vec3 diffuse = diff * normalize(plight[i].color);
        // Calculate attenuation
        vec3 dist = plight[i].position - vertex.position;
        float dist2 = dot(dist, dist);
        float attenuation = (plight[i].intensity / dist2);
        lighting += diffuse * attenuation;
    }
    for (uint i = 0u; i < directional_light_count; i++) {
        vec3 dir = dlight[i].direction;
        float diff = max(dot(-dir, normal), 0.0);
        vec3 diffuse = diff * dlight[i].color;
        lighting += diffuse;
    }
    lighting += ambient_color;
    out_color = vec4(lighting, 1.0) * color + ecolor;
}

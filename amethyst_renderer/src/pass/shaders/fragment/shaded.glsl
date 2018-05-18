// TODO: Needs documentation.

#version 150 core

layout (std140) uniform FragmentArgs {
    uint point_light_count;
    uint directional_light_count;
};

layout (std140) struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
};

layout (std140) uniform PointLights {
    PointLight plight[128];
};

struct DirectionalLight {
    vec3 color;
    vec3 direction;
};

layout (std140) uniform DirectionalLights {
    DirectionalLight dlight[16];
};

uniform vec3 ambient_color;
uniform vec3 camera_position;

uniform sampler2D albedo;
uniform sampler2D emission;

layout (std140) uniform AlbedoOffset {
    vec2 u_offset;
    vec2 v_offset;
} albedo_offset;

layout (std140) uniform EmissionOffset {
    vec2 u_offset;
    vec2 v_offset;
} emission_offset;

in VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
} vertex;

out vec4 out_color;

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
        PointLight light = plight[i];
        vec3 light_dir = normalize(light.position - vertex.position);
        float diff = max(dot(light_dir, normal), 0.0);
        vec3 diffuse = diff * normalize(light.color);
        // Calculate attenuation
        vec3 dist = light.position - vertex.position;
        float dist2 = dot(dist, dist);
        float attenuation = (light.intensity / dist2);
        lighting += attenuation;
    }
    for (uint i = 0u; i < directional_light_count; i++) {
        vec3 dir = dlight[i].direction;
        float diff = max(dot(-dir, normal), 0.0);
        vec3 diffuse = diff * dlight[i].color;
        lighting += diffuse;
    }
    lighting += ambient_color;
    out_color = vec4(lighting, 1.0) * color + ecolor;
    int i = 0;
    // out_color = vec4(plight[0].intensity / 100.0, 0.0, 0.0, 1.0);
}

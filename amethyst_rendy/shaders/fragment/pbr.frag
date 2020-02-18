#version 450

#include "header/math.frag"

#include "header/environment.frag"

layout(std140, set = 1, binding = 0) uniform Material {
    UvOffset uv_offset;
    float alpha_cutoff;
};

layout(set = 1, binding = 1) uniform sampler2D albedo;
layout(set = 1, binding = 2) uniform sampler2D emission;
layout(set = 1, binding = 3) uniform sampler2D normal;
layout(set = 1, binding = 4) uniform sampler2D metallic_roughness;
layout(set = 1, binding = 5) uniform sampler2D ambient_occlusion;
layout(set = 1, binding = 6) uniform sampler2D cavity;

layout(location = 0) in VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    float tang_handedness;
    vec2 tex_coord;
    vec4 color;
} vertex;

layout(location = 0) out vec4 out_color;


vec3 fresnel(float HdotV, vec3 fresnel_base) {
    return fresnel_base + (1.0 - fresnel_base) * pow(1.0 - HdotV, 5.0);
}

vec3 compute_light(vec3 attenuation,
                   vec3 light_color,
                   vec3 view_direction,
                   vec3 light_direction,
                   vec3 albedo,
                   vec3 normal,
                   float roughness2,
                   float metallic,
                   vec3 fresnel_base) {

    vec3 halfway = normalize(view_direction + light_direction);
    float normal_distribution = ggx_normal_distribution(normal, halfway, roughness2);

    float NdotV = max(dot(normal, view_direction), 0.0);
    float NdotL = max(dot(normal, light_direction), 0.0);
    float HdotV = max(dot(halfway, view_direction), 0.0);
    float geometry = ggx_geometry(NdotV, NdotL, roughness2);


    vec3 fresnel = fresnel(HdotV, fresnel_base);
    vec3 diffuse = vec3(1.0) - fresnel;
    diffuse *= 1.0 - metallic;

    vec3 nominator = normal_distribution * geometry * fresnel;
    float denominator = 4 * NdotV * NdotL + 0.0001;
    vec3 specular = nominator / denominator;

    vec3 resulting_light = (diffuse * albedo / PI + specular) * light_color * attenuation * NdotL;
    return resulting_light;
}

void main() {
    vec2 final_tex_coords   = tex_coords(vertex.tex_coord, uv_offset);
    vec4 albedo_alpha       = texture(albedo, final_tex_coords);
    float alpha             = albedo_alpha.a;
    if(alpha < alpha_cutoff) discard;

    vec3 albedo             = albedo_alpha.rgb;
    vec3 emission           = texture(emission, final_tex_coords).rgb;
    vec3 normal             = texture(normal, final_tex_coords).rgb;
    vec2 metallic_roughness = texture(metallic_roughness, final_tex_coords).bg;
    float ambient_occlusion = texture(ambient_occlusion, final_tex_coords).r;
    // TODO: Use cavity
    // float cavity            = texture(cavity, tex_coords(vertex.tex_coord, final_tex_coords).r;
    float metallic          = metallic_roughness.r;
    float roughness         = metallic_roughness.g;

    // normal conversion
    normal = normal * 2 - 1;

    float roughness2 = roughness * roughness;
    vec3 fresnel_base = mix(vec3(0.04), albedo, metallic);

    vec3 vertex_normal = normalize(vertex.normal);
    vec3 vertex_tangent = normalize(vertex.tangent - vertex_normal * dot(vertex_normal, vertex.tangent));
    vec3 vertex_bitangent = normalize(cross(vertex_normal, vertex_tangent) * vertex.tang_handedness);
    mat3 vertex_basis = mat3(vertex_tangent, vertex_bitangent, vertex_normal);
    normal = normalize(vertex_basis * normal);

    vec3 view_direction = normalize(camera_position - vertex.position);
    vec3 lighted = vec3(0.0);
    for (int i = 0; i < point_light_count; i++) {
        vec3 light_direction = normalize(plight[i].position - vertex.position);
        float attenuation = plight[i].intensity / dot(light_direction, light_direction);

        vec3 light = compute_light(vec3(attenuation),
                                   plight[i].color,
                                   view_direction,
                                   light_direction,
                                   albedo,
                                   normal,
                                   roughness2,
                                   metallic,
                                   fresnel_base);

        lighted += light;
    }

    for (int i = 0; i < directional_light_count; i++) {
        vec3 light_direction = -normalize(dlight[i].direction);
        float attenuation = dlight[i].intensity;

        vec3 light = compute_light(vec3(attenuation),
                                   dlight[i].color,
                                   view_direction,
                                   light_direction,
                                   albedo,
                                   normal,
                                   roughness2,
                                   metallic,
                                   fresnel_base);

        lighted += light;
    }

    for (int i = 0; i < spot_light_count; i++) {
        vec3 light_vec = slight[i].position - vertex.position;
        vec3 normalized_light_vec = normalize(light_vec);

        // The distance between the current fragment and the "core" of the light
        float light_length = length(light_vec);

        // The allowed "length", everything after this won't be lit.
        // Later on we are dividing by this range, so it can't be 0
        float range = max(slight[i].range, 0.00001);

        // get normalized range, so everything 0..1 could be lit, everything else can't.
        float normalized_range = light_length / max(0.00001, range);

        // The attenuation for the "range". If we would only consider this, we'd have a
        // point light instead, so we need to also check for the spot angle and direction.
        float range_attenuation = max(0.0, 1.0 - normalized_range);

        // this is actually the cosine of the angle, so it can be compared with the
        // "dotted" frag_angle below a lot cheaper.
        float spot_angle = max(slight[i].angle, 0.00001);
        vec3 spot_direction = normalize(slight[i].direction);
        float smoothness = 1.0 - slight[i].smoothness;

        // Here we check if the current fragment is within the "ring" of the spotlight.
        float frag_angle = dot(spot_direction, -normalized_light_vec);

        // so that the ring_attenuation won't be > 1
        frag_angle = max(frag_angle, spot_angle);

        // How much is this outside of the ring? (let's call it "rim")
        // Also smooth this out.
        float rim_attenuation = pow(max((1.0 - frag_angle) / (1.0 - spot_angle), 0.00001), smoothness);

        // How much is this inside the "ring"?
        float ring_attenuation = 1.0 - rim_attenuation;

        // combine the attenuations and intensity
        float attenuation = range_attenuation * ring_attenuation * slight[i].intensity;

        vec3 light = compute_light(vec3(attenuation),
                                   slight[i].color,
                                   view_direction,
                                   normalize(light_vec),
                                   albedo,
                                   normal,
                                   roughness2,
                                   metallic,
                                   fresnel_base);
        lighted += light;
    }

    vec3 ambient = ambient_color * albedo * ambient_occlusion;
    vec3 color = ambient + lighted + emission;

    out_color = vec4(color, alpha) * vertex.color;
}

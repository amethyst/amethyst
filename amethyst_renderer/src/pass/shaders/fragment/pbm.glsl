// TODO: Needs documentation.

#version 150 core

layout (std140) uniform FragmentArgs {
    int point_light_count;
    int directional_light_count;
    int spot_light_count;
};

struct PointLight {
    vec3 position;
    vec3 color;
    float pad; // Workaround for bug in mac's implementation of opengl (loads garbage when accessing members of structures in arrays with dynamic indices).
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

struct SpotLight {
    vec3 position;
    vec3 color;
    vec3 direction;
    float angle;
    float intensity;
    float range;
    float smoothness;
};

layout (std140) uniform SpotLights {
    SpotLight slight[128];
};

uniform vec3 ambient_color;
uniform vec3 camera_position;

uniform float alpha_cutoff;

uniform sampler2D albedo;
uniform sampler2D emission;
uniform sampler2D normal;
uniform sampler2D metallic;
uniform sampler2D roughness;
uniform sampler2D ambient_occlusion;
uniform sampler2D caveat;

layout (std140) uniform AlbedoOffset {
    vec2 u_offset;
    vec2 v_offset;
} albedo_offset;

layout (std140) uniform EmissionOffset {
    vec2 u_offset;
    vec2 v_offset;
} emission_offset;

layout (std140) uniform NormalOffset {
    vec2 u_offset;
    vec2 v_offset;
} normal_offset;

layout (std140) uniform MetallicOffset {
    vec2 u_offset;
    vec2 v_offset;
} metallic_offset;

layout (std140) uniform RoughnessOffset {
    vec2 u_offset;
    vec2 v_offset;
} roughness_offset;

layout (std140) uniform AmbientOcclusionOffset {
    vec2 u_offset;
    vec2 v_offset;
} ambient_occlusion_offset;

layout (std140) uniform CaveatOffset {
    vec2 u_offset;
    vec2 v_offset;
} caveat_offset;

in VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
    vec4 color;
} vertex;

out vec4 out_color;

const float PI = 3.14159265359;

float tex_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

vec2 tex_coords(vec2 coord, vec2 u, vec2 v) {
    return vec2(tex_coord(coord.x, u), tex_coord(coord.y, v));
}

float normal_distribution(vec3 N, vec3 H, float a) {
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return (a2 + 0.0000001) / denom;
}

float geometry(float NdotV, float NdotL, float r2) {
    float a1 = r2 + 1.0;
    float k = a1 * a1 / 8.0;
    float denom = NdotV * (1.0 - k) + k;
    float ggx1 = NdotV / denom;
    denom = NdotL * (1.0 - k) + k;
    float ggx2 = NdotL / denom;
    return ggx1 * ggx2;
}

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
    float normal_distribution = normal_distribution(normal, halfway, roughness2);

    float NdotV = max(dot(normal, view_direction), 0.0);
    float NdotL = max(dot(normal, light_direction), 0.0);
    float HdotV = max(dot(halfway, view_direction), 0.0);
    float geometry = geometry(NdotV, NdotL, roughness2);

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
    vec4 albedo_alpha       = texture(albedo, tex_coords(vertex.tex_coord, albedo_offset.u_offset, albedo_offset.v_offset)).rgba;

    float alpha             = albedo_alpha.a;
    if(alpha < alpha_cutoff) discard;

    vec3 albedo             = albedo_alpha.rgb;
    vec3 emission           = texture(emission, tex_coords(vertex.tex_coord, emission_offset.u_offset, emission_offset.v_offset)).rgb;
    vec3 normal             = texture(normal, tex_coords(vertex.tex_coord, normal_offset.u_offset, normal_offset.v_offset)).rgb;
    float metallic          = texture(metallic, tex_coords(vertex.tex_coord, metallic_offset.u_offset, metallic_offset.v_offset)).r;
    float roughness         = texture(roughness, tex_coords(vertex.tex_coord, roughness_offset.u_offset, roughness_offset.v_offset)).r;
    float ambient_occlusion = texture(ambient_occlusion, tex_coords(vertex.tex_coord, ambient_occlusion_offset.u_offset, ambient_occlusion_offset.v_offset)).r;
    float caveat            = texture(caveat, tex_coords(vertex.tex_coord, caveat_offset.u_offset, caveat_offset.v_offset)).r; // TODO: Use caveat

    // normal conversion
    normal = normal * 2 - 1;

    float roughness2 = roughness * roughness;
    vec3 fresnel_base = mix(vec3(0.04), albedo, metallic);

    vec3 vertex_normal = normalize(vertex.normal);
    vec3 vertex_tangent = normalize(vertex.tangent - vertex_normal * dot(vertex_normal, vertex.tangent));
    vec3 vertex_bitangent = normalize(cross(vertex_normal, vertex_tangent));
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
        float attenuation = 1.0;

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

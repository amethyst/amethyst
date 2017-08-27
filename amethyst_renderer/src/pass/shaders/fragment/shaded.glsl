// TODO: Needs documentation.

#version 150 core

layout (std140) uniform FragmentArgs {
    int point_light_count;
    int directional_light_count;
};

struct PointLight {
    vec4 position;
    vec4 color;
    float intensity;
    float radius;
    float smoothness;
    float _pad;
};

layout (std140) uniform PointLights {
    PointLight plight[512];
};

struct DirectionalLight {
    vec3 direction;
    vec4 color;
};

layout (std140) uniform DirectionalLights {
    DirectionalLight dlight[16];
};

uniform vec3 ambient_color;
uniform vec3 camera_position;

uniform sampler2D albedo;
uniform sampler2D emission;

in VertexData {
    vec4 position;
    vec3 normal;
    vec2 tex_coord;
} vertex;

out vec4 out_color;

void main() {
    vec4 color = texture(albedo, vertex.tex_coord);
    vec4 ecolor = texture(emission, vertex.tex_coord);
    vec4 lighting = vec4(0.0);
    vec4 normal = vec4(normalize(vertex.normal), 0.0);
    for (int i = 0; i < point_light_count; i++) {
        // Calculate diffuse light
        vec4 light_dir = normalize(plight[i].position - vertex.position);
        float diff = max(dot(light_dir, normal), 0.0);
        vec4 diffuse = diff * plight[i].color;
        // Calculate attenuation
        vec4 dist = plight[i].position - vertex.position;
        float dist2 = dot(dist, dist);
        float attenuation = (plight[i].intensity / dist2);
        lighting += diffuse * attenuation;
    }
    for (int i = 0; i < directional_light_count; i++) {
        vec4 dir = vec4(dlight[i].direction, 1.0);
        float diff = max(dot(-dir, normal), 0.0);
        vec4 diffuse = diff * dlight[i].color;
        lighting += diffuse;
    }
    lighting += vec4(ambient_color, 0.0);
    out_color = lighting * color + ecolor;
}

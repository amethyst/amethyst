// Environment shader definition.
// Set 0.
// Keep in sync with amethyst_rendy/src/submodules/environment.rs

struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
};

struct DirectionalLight {
    vec3 color;
    float intensity;
    vec3 direction;
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

layout(std140, set = 0, binding = 1) uniform Environment {
    vec3 ambient_color;
    vec3 camera_position; 
    int point_light_count;
    int directional_light_count;
    int spot_light_count;
};

layout(std140, set = 0, binding = 2) uniform PointLights {
    PointLight plight[128];
};

layout(std140, set = 0, binding = 3) uniform DirectionalLights {
    DirectionalLight dlight[16];
};

layout(std140, set = 0, binding = 4) uniform SpotLights {
    SpotLight slight[128];
};
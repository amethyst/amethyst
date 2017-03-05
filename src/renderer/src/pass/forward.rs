use gfx;
use gfx::traits::FactoryExt;

use pass;
use target::ColorBuffer;
pub use VertexPosNormal;

pub static VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    layout (std140) uniform cb_VertexArgs {
        uniform mat4 u_Proj;
        uniform mat4 u_View;
        uniform mat4 u_Model;
    };

    in vec3 a_Pos;
    in vec3 a_Normal;
    in vec2 a_TexCoord;

    out VertexData {
        vec4 Position;
        vec3 Normal;
        vec2 TexCoord;
    } v_Out;

    void main() {
        v_Out.Position = u_Model * vec4(a_Pos, 1.0);
        v_Out.Normal = mat3(u_Model) * a_Normal;
        v_Out.TexCoord = a_TexCoord;
        gl_Position = u_Proj * u_View * v_Out.Position;
    }
";

pub static FLAT_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform sampler2D t_Ka;

    in VertexData {
        vec4 Position;
        vec3 Normal;
        vec2 TexCoord;
    } v_In;

    out vec4 o_Color;

    void main() {
        o_Color = texture(t_Ka, v_In.TexCoord);
    }
";

pub static FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    layout (std140) uniform cb_FragmentArgs {
        int u_PointLightCount;
        int u_DirectionalLightCount;
    };

    struct PointLight {
        vec4 center;
        vec4 color;
        float intensity;
        float radius;
        float smoothness;
        float pad;
    };

    layout (std140) uniform u_PointLights {
        PointLight plight[512];
    };

    struct DirectionalLight {
        vec4 color;
        vec4 direction;
    };

    layout (std140) uniform u_DirectionalLights {
        DirectionalLight dlight[16];
    };

    uniform sampler2D t_Ka;
    uniform sampler2D t_Kd;
    uniform sampler2D t_Ks;
    uniform float f_Ns;
    uniform float f_Ambient;

    in VertexData {
        vec4 Position;
        vec3 Normal;
        vec2 TexCoord;
    } v_In;

    out vec4 o_Color;

    void main() {
        vec4 color = texture(t_Ka, v_In.TexCoord);
        vec4 kd = texture(t_Kd, v_In.TexCoord);
        vec4 ks = texture(t_Ks, v_In.TexCoord);
        vec4 lighting = vec4(0.0);
        vec4 normal = vec4(normalize(v_In.Normal), 0.0);

        for (int i = 0; i < u_PointLightCount; i++) {
            // Calculate diffuse light
            vec4 lightDir = normalize(plight[i].center - v_In.Position);
            float diff = max(dot(lightDir, normal), 0.0);
            vec4 diffuse = diff * plight[i].color * kd;

            // Calculate specular light. Uses Blinn-Phong model
            // for specular highlights.
            vec4 viewDir = normalize(-v_In.Position);
            vec4 reflectDir = reflect(-lightDir, normal);
            vec4 halfwayDir = normalize(lightDir + viewDir);
            float spec = pow(max(dot(normal, halfwayDir), 0.0), f_Ns);
            vec4 specular = spec * plight[i].color * ks;

            // Calculate attenuation
            float dist = length(plight[i].center - v_In.Position);
            float smoothness = plight[i].smoothness;
            float window = 1.0 - pow(dist, smoothness) / pow(plight[i].radius, smoothness);
            float attenuation = (plight[i].intensity / dist) * pow(clamp(window, 0.0, 1.0), 2.0);

            lighting += attenuation * (diffuse + specular);
        }

        for (int i = 0; i < u_DirectionalLightCount; i++) {
            vec4 dir = dlight[i].direction;
            float diff = max(dot(-dir, normal), 0.0);
            vec4 diffuse = diff * dlight[i].color * kd;

            vec4 viewDir = normalize(-v_In.Position);
            vec4 reflectDir = reflect(-dir, normal);
            vec4 halfwayDir = normalize(dir + viewDir);
            float spec = pow(max(dot(normal, halfwayDir), 0.0), f_Ns);
            vec4 specular = spec * dlight[i].color * ks;

            lighting += diffuse + specular;
        }

        color *= f_Ambient * color + lighting;
        o_Color = color;
    }
";

pub static WIREFRAME_GEOMETRY_SRC: &'static [u8] = b"
    #version 150 core

    layout(triangles) in;
    layout(line_strip, max_vertices=4) out;

    in VertexData {
        vec4 Position;
        vec3 Normal;
        vec2 TexCoord;
    } v_In[];

    out VertexData {
        vec4 Position;
        vec3 Normal;
        vec2 TexCoord;
    } v_Out;

    void main() {
        v_Out.Position = v_In[0].Position;
        v_Out.Normal = v_In[0].Normal;
        v_Out.TexCoord = v_In[0].TexCoord;
        gl_Position = gl_in[0].gl_Position;
        EmitVertex();
        v_Out.Position = v_In[1].Position;
        v_Out.Normal = v_In[1].Normal;
        v_Out.TexCoord = v_In[1].TexCoord;
        gl_Position = gl_in[1].gl_Position;
        EmitVertex();
        v_Out.Position = v_In[2].Position;
        v_Out.Normal = v_In[2].Normal;
        v_Out.TexCoord = v_In[0].TexCoord;
        gl_Position = gl_in[2].gl_Position;
        EmitVertex();
        v_Out.Position = v_In[0].Position;
        v_Out.Normal = v_In[0].Normal;
        v_Out.TexCoord = v_In[0].TexCoord;
        gl_Position = gl_in[0].gl_Position;
        EmitVertex();
        EndPrimitive();
    }
";

pub type GFormat = [f32; 4];

gfx_defines!(
    // Necessary for these to be `[f32; 4]` in order for shader
    // transforms to work correctly, even though attenuation/center
    // are really `[f32; 3]`.
    constant PointLight {
        center: [f32; 4] = "center",
        color: [f32; 4] = "color",
        intensity: f32 = "intensity",
        radius: f32 = "radius",
        smoothness: f32 = "smoothness",
        _pad: f32 = "pad",
    }

    constant DirectionalLight {
        color: [f32; 4] = "color",
        direction: [f32; 4] = "direction",
    }

    constant VertexArgs {
        proj: [[f32; 4]; 4] = "u_Proj",
        view: [[f32; 4]; 4] = "u_View",
        model: [[f32; 4]; 4] = "u_Model",
    }

    constant FragmentArgs {
        point_light_count: i32 = "u_PointLightCount",
        directional_light_count: i32 = "u_DirectionalLightCount",
    }

    pipeline flat {
        vbuf: gfx::VertexBuffer<VertexPosNormal> = (),
        vertex_args: gfx::ConstantBuffer<VertexArgs> = "cb_VertexArgs",
        fragment_args: gfx::ConstantBuffer<FragmentArgs> = "cb_FragmentArgs",
        out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Color",
        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
        ka: gfx::TextureSampler<[f32; 4]> = "t_Ka",
        kd: gfx::TextureSampler<[f32; 4]> = "t_Kd",
    }

    pipeline shaded {
        vbuf: gfx::VertexBuffer<VertexPosNormal> = (),
        vertex_args: gfx::ConstantBuffer<VertexArgs> = "cb_VertexArgs",
        fragment_args: gfx::ConstantBuffer<FragmentArgs> = "cb_FragmentArgs",
        point_lights: gfx::ConstantBuffer<PointLight> = "u_PointLights",
        dir_lights: gfx::ConstantBuffer<DirectionalLight> = "u_DirectionalLights",
        out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Color",
        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
        ka: gfx::TextureSampler<[f32; 4]> = "t_Ka",
        kd: gfx::TextureSampler<[f32; 4]> = "t_Kd",
        ks: gfx::TextureSampler<[f32; 4]> = "t_Ks",
        ns: gfx::Global<f32> = "f_Ns",
        ambient: gfx::Global<f32> = "f_Ambient",
    }

    pipeline wireframe {
        vbuf: gfx::VertexBuffer<VertexPosNormal> = (),
        vertex_args: gfx::ConstantBuffer<VertexArgs> = "cb_VertexArgs",
        out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Color",
        ka: gfx::TextureSampler<[f32; 4]> = "t_Ka",
        kd: gfx::TextureSampler<[f32; 4]> = "t_Kd",
    }
);

/// Handles rendering fragments with no shading
pub struct DrawFlat<R: gfx::Resources> {
    vertex: gfx::handle::Buffer<R, VertexArgs>,
    fragment: gfx::handle::Buffer<R, FragmentArgs>,
    ka: ::ConstantColorTexture<R>,
    kd: ::ConstantColorTexture<R>,
    pso: gfx::pso::PipelineState<R, flat::Meta>,
    sampler: gfx::handle::Sampler<R>,
}

impl<R: gfx::Resources> DrawFlat<R> {
    pub fn new<F>(factory: &mut F) -> DrawFlat<R>
        where R: gfx::Resources,
              F: gfx::Factory<R>
    {
        let vertex = factory.create_constant_buffer(1);
        let fragment = factory.create_constant_buffer(1);
        let pso = factory.create_pipeline_simple(VERTEX_SRC, FLAT_FRAGMENT_SRC, flat::new())
            .expect("Could not create PSO for `DrawFlat`!");

        let sampler =
            factory.create_sampler(gfx::texture::SamplerInfo::new(gfx::texture::FilterMethod::Scale,
                                                                  gfx::texture::WrapMode::Clamp));

        DrawFlat {
            vertex: vertex,
            fragment: fragment,
            ka: ::ConstantColorTexture::new(factory),
            kd: ::ConstantColorTexture::new(factory),
            pso: pso,
            sampler: sampler,
        }
    }
}

impl<R> pass::Pass<R> for DrawFlat<R>
where R: gfx::Resources
{
    type Arg = pass::DrawFlat;
    type Target = ColorBuffer<R>;

    fn apply<C>(&self,
                _: &pass::DrawFlat,
                target: &ColorBuffer<R>,
                _: &::Pipeline,
                fragments: &[::Fragment<R>],
                scene: &::Scene,
                encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        // every entity gets drawn
        for e in fragments {
            encoder.update_constant_buffer(
                &self.vertex,
                &VertexArgs {
                    proj: scene.camera.proj,
                    view: scene.camera.view,
                    model: e.transform,
                }
            );

            encoder.update_constant_buffer(
                &self.fragment,
                &FragmentArgs {
                    point_light_count: 0,
                    directional_light_count: 0,
                }
            );

            let ka = e.ka.to_view(&self.ka, encoder);
            let kd = e.kd.to_view(&self.kd, encoder);

            encoder.draw(&e.slice,
                         &self.pso,
                         &flat::Data {
                             vbuf: e.buffer.clone(),
                             vertex_args: self.vertex.clone(),
                             fragment_args: self.fragment.clone(),
                             out_ka: target.color.clone(),
                             out_depth: target.output_depth.clone(),
                             ka: (ka, self.sampler.clone()),
                             kd: (kd, self.sampler.clone()),
                         });
        }
    }
}


/// Handles rendering fragments with shading
pub struct DrawShaded<R: gfx::Resources> {
    vertex: gfx::handle::Buffer<R, VertexArgs>,
    fragment: gfx::handle::Buffer<R, FragmentArgs>,
    point_lights: gfx::handle::Buffer<R, PointLight>,
    dir_lights: gfx::handle::Buffer<R, DirectionalLight>,
    pso: gfx::pso::PipelineState<R, shaded::Meta>,
    sampler: gfx::handle::Sampler<R>,
    ka: ::ConstantColorTexture<R>,
    kd: ::ConstantColorTexture<R>,
    ks: ::ConstantColorTexture<R>,
}

impl<R: gfx::Resources> DrawShaded<R> {
    pub fn new<F>(factory: &mut F) -> DrawShaded<R>
        where R: gfx::Resources,
              F: gfx::Factory<R>
    {
        let point_lights = factory.create_constant_buffer(512);
        let dir_lights = factory.create_constant_buffer(16);
        let vertex = factory.create_constant_buffer(1);
        let fragment = factory.create_constant_buffer(1);
        let pso = factory.create_pipeline_simple(VERTEX_SRC, FRAGMENT_SRC, shaded::new())
            .expect("Could not create PSO for `DrawShaded`!");

        let sampler =
            factory.create_sampler(gfx::texture::SamplerInfo::new(gfx::texture::FilterMethod::Scale,
                                                               gfx::texture::WrapMode::Clamp));

        DrawShaded {
            vertex: vertex,
            fragment: fragment,
            point_lights: point_lights,
            dir_lights: dir_lights,
            pso: pso,
            ka: ::ConstantColorTexture::new(factory),
            kd: ::ConstantColorTexture::new(factory),
            ks: ::ConstantColorTexture::new(factory),
            sampler: sampler,
        }
    }
}


fn pad(x: [f32; 3]) -> [f32; 4] {
    [x[0], x[1], x[2], 0.]
}

impl<R> pass::Pass<R> for DrawShaded<R>
    where R: gfx::Resources
{
    type Arg = pass::DrawShaded;
    type Target = ColorBuffer<R>;

    fn apply<C>(&self,
                _: &pass::DrawShaded,
                target: &ColorBuffer<R>,
                _: &::Pipeline,
                fragments: &[::Fragment<R>],
                scene: &::Scene,
                encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {

        // Add point lights to scene
        let point_lights: Vec<_> = scene.point_lights
            .iter()
            .map(|l| {
                PointLight {
                    color: l.color,
                    center: pad(l.center),
                    intensity: l.intensity,
                    radius: l.radius,
                    smoothness: l.smoothness,
                    _pad: 0.0,
                }
            })
            .collect();
        encoder.update_buffer(&self.point_lights, &point_lights[..], 0).unwrap();

        // Add directional lights to scene
        let dir_lights: Vec<_> = scene.directional_lights.iter()
            .map(|l| {
                DirectionalLight {
                    color: l.color,
                    direction: pad(l.direction),
                }
            })
            .collect();
        encoder.update_buffer(&self.dir_lights, &dir_lights[..], 0).unwrap();

        // Draw every entity
        for e in fragments {
            encoder.update_constant_buffer(
                &self.vertex,
                &VertexArgs {
                    proj: scene.camera.proj,
                    view: scene.camera.view,
                    model: e.transform,
                }
            );

            encoder.update_constant_buffer(
                &self.fragment,
                &FragmentArgs {
                    point_light_count: scene.point_lights.len() as i32,
                    directional_light_count: scene.directional_lights.len() as i32,
                }
            );

            let ka = e.ka.to_view(&self.ka, encoder);
            let kd = e.kd.to_view(&self.kd, encoder);
            let ks = e.ks.to_view(&self.ks, encoder);

            encoder.draw(&e.slice,
                         &self.pso,
                         &shaded::Data {
                             vbuf: e.buffer.clone(),
                             fragment_args: self.fragment.clone(),
                             vertex_args: self.vertex.clone(),
                             point_lights: self.point_lights.clone(),
                             dir_lights: self.dir_lights.clone(),
                             out_ka: target.color.clone(),
                             out_depth: target.output_depth.clone(),
                             ka: (ka, self.sampler.clone()),
                             kd: (kd, self.sampler.clone()),
                             ks: (ks, self.sampler.clone()),
                             ns: e.ns,
                             ambient: scene.ambient_light,
                         });
        }
    }
}


/// Handles rendering fragments as wireframe objects
pub struct Wireframe<R: gfx::Resources> {
    vertex: gfx::handle::Buffer<R, VertexArgs>,
    pso: gfx::pso::PipelineState<R, wireframe::Meta>,
    sampler: gfx::handle::Sampler<R>,
    ka: ::ConstantColorTexture<R>,
    kd: ::ConstantColorTexture<R>,
}

impl<R: gfx::Resources> Wireframe<R> {
    pub fn new<F>(factory: &mut F) -> Wireframe<R>
        where F: gfx::Factory<R>
    {
        let vs = factory.create_shader_vertex(VERTEX_SRC).unwrap();
        let gs = factory.create_shader_geometry(WIREFRAME_GEOMETRY_SRC).unwrap();
        let fs = factory.create_shader_pixel(FLAT_FRAGMENT_SRC).unwrap();
        let vertex = factory.create_constant_buffer(1);
        let pso = factory.create_pipeline_state(&gfx::ShaderSet::Geometry(vs, gs, fs),
                                   gfx::Primitive::TriangleList,
                                   gfx::state::Rasterizer::new_fill(),
                                   wireframe::new())
            .expect("Could not create PSO for `Wireframe`!");

        let sampler =
            factory.create_sampler(gfx::texture::SamplerInfo::new(gfx::texture::FilterMethod::Scale,
                                                               gfx::texture::WrapMode::Clamp));

        Wireframe {
            vertex: vertex,
            pso: pso,
            sampler: sampler,
            ka: ::ConstantColorTexture::new(factory),
            kd: ::ConstantColorTexture::new(factory),
        }
    }
}

impl<R> pass::Pass<R> for Wireframe<R>
    where R: gfx::Resources
{
    type Arg = pass::Wireframe;
    type Target = ColorBuffer<R>;

    fn apply<C>(&self,
                _: &pass::Wireframe,
                target: &ColorBuffer<R>,
                _: &::Pipeline,
                fragments: &[::Fragment<R>],
                scene: &::Scene,
                encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {

        // every entity gets drawn
        for e in fragments {
            encoder.update_constant_buffer(
                &self.vertex,
                &VertexArgs {
                    proj: scene.camera.proj,
                    view: scene.camera.view,
                    model: e.transform,
                }
            );

            let ka = e.ka.to_view(&self.ka, encoder);
            let kd = e.kd.to_view(&self.kd, encoder);

            encoder.draw(&e.slice,
                         &self.pso,
                         &wireframe::Data {
                             vbuf: e.buffer.clone(),
                             vertex_args: self.vertex.clone(),
                             out_ka: target.color.clone(),
                             ka: (ka, self.sampler.clone()),
                             kd: (kd, self.sampler.clone()),
                         });
        }
    }
}

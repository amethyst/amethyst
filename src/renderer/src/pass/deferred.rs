use cgmath::{Matrix4, SquareMatrix};
use gfx;
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;

use ConstantColorTexture;
use pass;
pub use target::{ColorFormat, GeometryBuffer};

gfx_vertex_struct!(Vertex {
    pos: [i32; 2] = "a_Pos",
    tex_coord: [i32; 2] = "a_TexCoord",
});

pub struct Clear;

impl<R> pass::Pass<R> for Clear
    where R: gfx::Resources
{
    type Arg = pass::Clear;
    type Target = GeometryBuffer<R>;

    fn apply<C>(&self,
                c: &pass::Clear,
                target: &GeometryBuffer<R>,
                _: &::Pipeline,
                _: &::Scene<R>,
                encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        encoder.clear(&target.normal, [0.; 4]);
        encoder.clear(&target.ka, c.color);
        encoder.clear(&target.kd, c.color);
        encoder.clear(&target.ks, c.color);
        encoder.clear_depth(&target.depth, 1.0);
    }
}

pub static DRAW_VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    layout (std140) uniform cb_VertexArgs {
        uniform mat4 u_Proj;
        uniform mat4 u_View;
        uniform mat4 u_Model;
    };

    in vec3 a_Normal;
    in vec3 a_Pos;
    in vec2 a_TexCoord;

    out vec3 v_Normal;
    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_TexCoord;
        v_Normal = mat3(u_Model) * a_Normal;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
";

pub static DRAW_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform sampler2D t_Ka;
    uniform sampler2D t_Kd;
    uniform sampler2D t_Ks;

    in vec3 v_Normal;
    in vec2 v_TexCoord;

    out vec4 o_Ka;
    out vec4 o_Kd;
    out vec4 o_Ks;
    out vec4 o_Normal;

    void main() {
        o_Ka = texture(t_Ka, v_TexCoord);
        o_Kd = texture(t_Kd, v_TexCoord);
        o_Ks = texture(t_Ks, v_TexCoord);
        o_Normal = vec4(normalize(v_Normal), 0.);
    }
";

pub static DEPTH_VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    layout (std140) uniform cb_VertexArgs {
        uniform mat4 u_Proj;
        uniform mat4 u_View;
        uniform mat4 u_Model;
    };

    in vec3 a_Pos;

    void main() {
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
";

pub static DEPTH_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    void main() {
    }
";

pub static LIGHT_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    layout (std140) uniform u_FragmentLightArgs {
        mat4 u_Proj;
        mat4 u_InvViewProj;
        vec4 u_Viewport;
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

    struct DirectionalLight {
        vec4 color;
        vec4 direction;
    };

    layout (std140) uniform u_PointLights {
        PointLight plight[128];
    };

    layout (std140) uniform u_DirectionalLights {
        DirectionalLight dlight[16];
    };

    uniform sampler2D t_Ka;
    uniform sampler2D t_Kd;
    uniform sampler2D t_Ks;
    uniform float f_Ns;
    uniform sampler2D t_Depth;
    uniform sampler2D t_Normal;
    uniform float f_Ambient;

    in vec2 v_TexCoord;
    out vec4 o_Color;

    vec4 calc_pos_from_window(vec3 window_space) {
        vec2 depthrange = vec2(0., 1.);
        vec3 ndc_pos;
        ndc_pos.xy = ((2.0 * window_space.xy) - (2.0 * u_Viewport.xy)) / (u_Viewport.zw) - 1;
        ndc_pos.z = (2.0 * window_space.z - depthrange.x - depthrange.y) /
                   (depthrange.y - depthrange.x);

        vec4 clip_pose;
        clip_pose.w = u_Proj[3][2] / (ndc_pos.z - (u_Proj[2][2] / u_Proj[2][3]));
        clip_pose.xyz = ndc_pos * clip_pose.w;

        return u_InvViewProj * clip_pose;
    }

    void main() {
        float depth = texture(t_Depth, v_TexCoord).x;
        vec4 ka = texture(t_Ka, v_TexCoord);
        vec4 kd = texture(t_Kd, v_TexCoord);
        vec4 ks = texture(t_Ks, v_TexCoord);
        vec4 normal = texture(t_Normal, v_TexCoord);

        vec4 pos = calc_pos_from_window(vec3(gl_FragCoord.xy, depth));

        vec4 lighting = vec4(0.0);

        for (int i = 0; i < u_PointLightCount; i++) {
            // Calculate diffuse light
            vec4 lightDir = normalize(plight[i].center - pos);
            float diff = max(dot(lightDir, normal), 0.0);
            vec4 diffuse = diff * plight[i].color * kd;

            // Calculate specular light
            vec4 viewDir = normalize(-gl_FragCoord);
            vec4 reflectDir = reflect(-lightDir, normal);
            vec4 halfwayDir = normalize(lightDir + viewDir);
            float spec = pow(max(dot(normal, halfwayDir), 0.0), f_Ns);
            vec4 specular = spec * plight[i].color * ks;

            // Calculate attenuation
            float dist = length(plight[i].center - pos);
            float smoothness = plight[i].smoothness;
            float window = 1.0 - pow(dist, smoothness) / pow(plight[i].radius, smoothness);
            float attenuation = (plight[i].intensity / dist) * pow(clamp(window, 0.0, 1.0), 2.0);

            lighting += attenuation * (diffuse + specular);
        }

        for (int i = 0; i < u_DirectionalLightCount; i++) {
            vec4 dir = dlight[i].direction;
            float diff = max(dot(-dir, normal), 0.0);
            vec4 diffuse = diff * dlight[i].color * kd;

            vec4 viewDir = normalize(-gl_FragCoord);
            vec4 reflectDir = reflect(-dir, normal);
            vec4 halfwayDir = normalize(dir + viewDir);
            float spec = pow(max(dot(normal, halfwayDir), 0.0), f_Ns);
            vec4 specular = spec * dlight[i].color * ks;

            lighting += diffuse + specular;
        }

        o_Color = ka * f_Ambient + lighting;
    }
";

gfx_defines!(
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

    constant FragmentLightArgs {
        proj: [[f32; 4]; 4] = "u_Proj",
        inv_view_proj: [[f32; 4]; 4] = "u_InvViewProj",
        viewport: [f32; 4] = "u_Viewport",
        point_light_count: i32 = "u_PointLightCount",
        directional_light_count: i32 = "u_DirectionalLightCount",
    }

    constant VertexArgs {
        proj: [[f32; 4]; 4] = "u_Proj",
        view: [[f32; 4]; 4] = "u_View",
        model: [[f32; 4]; 4] = "u_Model",
    }

    constant FragmentArgs {
        ka: [f32; 4] = "u_Ka",
        kd: [f32; 4] = "u_Kd",
        ks: [f32; 4] = "u_Ks",
    }

    pipeline light {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        ka: gfx::TextureSampler<[f32; 4]> = "t_Ka",
        kd: gfx::TextureSampler<[f32; 4]> = "t_Kd",
        ks: gfx::TextureSampler<[f32; 4]> = "t_Ks",
        ns: gfx::Global<f32> = "f_Ns",
        ambient: gfx::Global<f32> = "f_Ambient",
        normal: gfx::TextureSampler<[f32; 4]> = "t_Normal",
        depth: gfx::TextureSampler<f32> = "t_Depth",
        out: gfx::BlendTarget<ColorFormat> = ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::MULTIPLY),
        point_lights: gfx::ConstantBuffer<PointLight> = "u_PointLights",
        directional_lights: gfx::ConstantBuffer<DirectionalLight> = "u_DirectionalLights",
        fragment_args: gfx::ConstantBuffer<FragmentLightArgs> = "u_FragmentLightArgs",
    }

    pipeline draw {
        ka: gfx::TextureSampler<[f32; 4]> = "t_Ka",
        kd: gfx::TextureSampler<[f32; 4]> = "t_Kd",
        ks: gfx::TextureSampler<[f32; 4]> = "t_Ks",
        vbuf: gfx::VertexBuffer<::VertexPosNormal> = (),
        vertex_args: gfx::ConstantBuffer<VertexArgs> = "cb_VertexArgs",
        fragment_args: gfx::ConstantBuffer<FragmentArgs> = "cb_FragmentArgs",
        out_normal: gfx::RenderTarget<[f32; 4]> = "o_Normal",
        out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Ka",
        out_kd: gfx::RenderTarget<gfx::format::Rgba8> = "o_Kd",
        out_ks: gfx::RenderTarget<gfx::format::Rgba8> = "o_Ks",
        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline depth {
        vbuf: gfx::VertexBuffer<::VertexPosNormal> = (),
        vertex_args: gfx::ConstantBuffer<VertexArgs> = "cb_VertexArgs",
        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
);

pub struct DrawPass<R: gfx::Resources> {
    vertex: gfx::handle::Buffer<R, VertexArgs>,
    fragment: gfx::handle::Buffer<R, FragmentArgs>,
    pso: gfx::PipelineState<R, draw::Meta>,
    ka: ConstantColorTexture<R>,
    kd: ConstantColorTexture<R>,
    ks: ConstantColorTexture<R>,
    sampler: gfx::handle::Sampler<R>,
}

impl<R: gfx::Resources> DrawPass<R> {
    pub fn new<F>(factory: &mut F) -> DrawPass<R>
        where F: gfx::Factory<R>
    {
        let sampler =
            factory.create_sampler(gfx::texture::SamplerInfo::new(gfx::texture::FilterMethod::Scale,
                                                               gfx::texture::WrapMode::Clamp));

        DrawPass {
            vertex: factory.create_constant_buffer(1),
            fragment: factory.create_constant_buffer(1),
            pso: factory.create_pipeline_simple(DRAW_VERTEX_SRC, DRAW_FRAGMENT_SRC, draw::new())
                .unwrap(),
            ka: ConstantColorTexture::new(factory),
            kd: ConstantColorTexture::new(factory),
            ks: ConstantColorTexture::new(factory),
            sampler: sampler,
        }
    }
}

impl<R> pass::Pass<R> for DrawPass<R>
    where R: gfx::Resources
{
    type Arg = pass::DrawFlat;
    type Target = GeometryBuffer<R>;

    fn apply<C>(&self,
                _: &pass::DrawFlat,
                target: &GeometryBuffer<R>,
                _: &::Pipeline,
                scene: &::Scene<R>,
                encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        // every entity gets drawn
        for f in &scene.fragments {
            encoder.update_constant_buffer(&self.vertex,
                                           &VertexArgs {
                                               proj: scene.camera.proj,
                                               view: scene.camera.view,
                                               model: f.transform,
                                           });

            let ka = f.ka.to_view(&self.ka, encoder);
            let kd = f.kd.to_view(&self.kd, encoder);
            let ks = f.ks.to_view(&self.ks, encoder);

            encoder.draw(&f.slice,
                         &self.pso,
                         &draw::Data {
                             fragment_args: self.fragment.clone(),
                             vertex_args: self.vertex.clone(),
                             vbuf: f.buffer.clone(),
                             out_normal: target.normal.clone(),
                             out_ka: target.ka.clone(),
                             out_kd: target.kd.clone(),
                             out_ks: target.ks.clone(),
                             out_depth: target.depth.clone(),
                             ka: (ka, self.sampler.clone()),
                             kd: (kd, self.sampler.clone()),
                             ks: (ks, self.sampler.clone()),
                         });
        }
    }
}

pub struct DepthPass<R: gfx::Resources> {
    vertex: gfx::handle::Buffer<R, VertexArgs>,
    pso: gfx::PipelineState<R, depth::Meta>,
}

impl<R: gfx::Resources> DepthPass<R> {
    pub fn new<F>(factory: &mut F) -> DepthPass<R>
        where F: gfx::Factory<R>
    {
        DepthPass {
            vertex: factory.create_constant_buffer(1),
            pso: factory.create_pipeline_simple(DEPTH_VERTEX_SRC, DEPTH_FRAGMENT_SRC, depth::new())
                .unwrap(),
        }
    }
}

impl<R> ::Pass<R> for DepthPass<R>
    where R: gfx::Resources
{
    type Arg = pass::DepthPass;
    type Target = GeometryBuffer<R>;

    fn apply<C>(&self,
                _: &pass::DepthPass,
                target: &GeometryBuffer<R>,
                _: &::Pipeline,
                scene: &::Scene<R>,
                encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        // every entity gets rendered into the depth layer
        // not touching all other layers in Gbuffer
        for f in &scene.fragments {
            encoder.update_constant_buffer(&self.vertex,
                                           &VertexArgs {
                                               proj: scene.camera.proj,
                                               view: scene.camera.view,
                                               model: f.transform,
                                           });

            encoder.draw(&f.slice,
                         &self.pso,
                         &depth::Data {
                             vertex_args: self.vertex.clone(),
                             vbuf: f.buffer.clone(),
                             out_depth: target.depth.clone(),
                         });
        }
    }
}

gfx_pipeline!( blit {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    source: gfx::TextureSampler<[f32; 4]> = "t_Source",
    out: gfx::RenderTarget<ColorFormat> = "o_Color",
});

pub static BLIT_VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    in ivec2 a_Pos;
    in ivec2 a_TexCoord;
    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = vec4(a_Pos, 0.0, 1.0);
    }
";

pub static BLIT_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform sampler2D t_Source;

    in vec2 v_TexCoord;
    out vec4 o_Color;

    void main() {
        o_Color = texture(t_Source, v_TexCoord);
    }
";


fn create_screen_fill_triangle<F, R>(factory: &mut F) -> (Buffer<R, Vertex>, gfx::Slice<R>)
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    let vertex_data = [Vertex {
                           pos: [-3, -1],
                           tex_coord: [-1, 0],
                       },
                       Vertex {
                           pos: [1, -1],
                           tex_coord: [1, 0],
                       },
                       Vertex {
                           pos: [1, 3],
                           tex_coord: [1, 2],
                       }];

    let buffer = factory.create_vertex_buffer(&vertex_data);
    let slice = gfx::Slice::new_match_vertex_buffer(&buffer);
    (buffer, slice)
}

pub struct BlitLayer<R: gfx::Resources> {
    buffer: Buffer<R, Vertex>,
    slice: gfx::Slice<R>,
    sampler: gfx::handle::Sampler<R>,
    pso: gfx::pso::PipelineState<R, blit::Meta>,
}

impl<R> BlitLayer<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F) -> BlitLayer<R>
        where F: gfx::Factory<R>
    {
        let (buffer, slice) = create_screen_fill_triangle(factory);

        let sampler =
            factory.create_sampler(gfx::texture::SamplerInfo::new(gfx::texture::FilterMethod::Scale,
                                                               gfx::texture::WrapMode::Clamp));

        BlitLayer {
            slice: slice,
            buffer: buffer,
            sampler: sampler,
            pso: factory.create_pipeline_simple(BLIT_VERTEX_SRC, BLIT_FRAGMENT_SRC, blit::new())
                .unwrap(),
        }
    }
}

impl<R> pass::Pass<R> for BlitLayer<R>
    where R: gfx::Resources
{
    type Arg = pass::BlitLayer;
    type Target = ::target::ColorBuffer<R>;

    fn apply<C>(&self,
                arg: &pass::BlitLayer,
                target: &::target::ColorBuffer<R>,
                pipeline: &::Pipeline,
                _: &::Scene<R>,
                encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        let src = &pipeline.targets[&arg.gbuffer];
        let src = src.downcast_ref::<GeometryBuffer<R>>().unwrap();

        let layer = match arg.layer.as_ref() {
            "ka" => src.texture_ka.clone(),
            "kd" => src.texture_kd.clone(),
            "ks" => src.texture_ks.clone(),
            "normal" => src.texture_normal.clone(),
            x => panic!("Unsupported layer {}", x),
        };

        encoder.draw(&self.slice,
                     &self.pso,
                     &blit::Data {
                         vbuf: self.buffer.clone(),
                         source: (layer, self.sampler.clone()),
                         out: target.color.clone(),
                     });
    }
}

pub struct LightingPass<R: gfx::Resources> {
    buffer: Buffer<R, Vertex>,
    point_lights: Buffer<R, PointLight>,
    directional_lights: Buffer<R, DirectionalLight>,
    fragment_args: Buffer<R, FragmentLightArgs>,
    slice: gfx::Slice<R>,
    sampler: gfx::handle::Sampler<R>,
    pso: gfx::pso::PipelineState<R, light::Meta>,
}

fn pad(x: [f32; 3]) -> [f32; 4] {
    [x[0], x[1], x[2], 0.]
}

impl<R: gfx::Resources> LightingPass<R> {
    pub fn new<F>(factory: &mut F) -> LightingPass<R>
        where F: gfx::Factory<R>
    {
        let (buffer, slice) = create_screen_fill_triangle(factory);
        let sampler =
            factory.create_sampler(gfx::texture::SamplerInfo::new(gfx::texture::FilterMethod::Scale,
                                                               gfx::texture::WrapMode::Clamp));

        let point_lights = factory.create_constant_buffer(128);
        let directional_lights = factory.create_constant_buffer(16);
        let fragment_args = factory.create_constant_buffer(1);
        LightingPass {
            point_lights: point_lights,
            directional_lights: directional_lights,
            buffer: buffer,
            slice: slice,
            sampler: sampler,
            fragment_args: fragment_args,
            pso: factory.create_pipeline_simple(BLIT_VERTEX_SRC, LIGHT_FRAGMENT_SRC, light::new())
                .unwrap(),
        }
    }
}

impl<R> pass::Pass<R> for LightingPass<R>
    where R: gfx::Resources
{
    type Arg = pass::Lighting;
    type Target = ::target::ColorBuffer<R>;

    fn apply<C>(&self,
                arg: &pass::Lighting,
                target: &::target::ColorBuffer<R>,
                pipeline: &::Pipeline,
                scene: &::Scene<R>,
                encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        let src = &pipeline.targets[&arg.gbuffer];
        let src = src.downcast_ref::<GeometryBuffer<R>>().unwrap();
        let (w, h, _, _) = src.kd.get_dimensions();

        let inv_view_proj = Matrix4::from(scene.camera.view).invert().unwrap() *
                            Matrix4::from(scene.camera.proj).invert().unwrap();

        // Add lighting to scene in chunks of 128 lights at a time
        // TODO: Why chunked?
        for chunk in scene.point_lights.chunks(128) {
            let point_lights: Vec<_> = chunk.iter()
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

            // Add directional lights to scene
            let directional_lights: Vec<_> = scene.directional_lights
                .iter()
                .map(|l| {
                    DirectionalLight {
                        color: l.color,
                        direction: pad(l.direction),
                    }
                })
                .collect();

            encoder.update_constant_buffer(&self.fragment_args,
                                           &FragmentLightArgs {
                                               inv_view_proj: inv_view_proj.into(),
                                               proj: scene.camera.proj,
                                               viewport: [0., 0., w as f32, h as f32],
                                               point_light_count: point_lights.len() as i32,
                                               directional_light_count: directional_lights.len() as
                                                                        i32,
                                           });

            encoder.update_buffer(&self.point_lights, &point_lights[..], 0).unwrap();
            encoder.update_buffer(&self.directional_lights, &directional_lights[..], 0).unwrap();

            encoder.draw(&self.slice,
                         &self.pso,
                         &light::Data {
                             vbuf: self.buffer.clone(),
                             ka: (src.texture_ka.clone(), self.sampler.clone()),
                             kd: (src.texture_kd.clone(), self.sampler.clone()),
                             ks: (src.texture_ks.clone(), self.sampler.clone()),
                             ns: 16.0, // TODO: Remove this hardcoded value, requires support for different materials
                             ambient: scene.ambient_light,
                             normal: (src.texture_normal.clone(), self.sampler.clone()),
                             depth: (src.texture_depth.clone(), self.sampler.clone()),
                             out: target.color.clone(),
                             fragment_args: self.fragment_args.clone(),
                             point_lights: self.point_lights.clone(),
                             directional_lights: self.directional_lights.clone(),
                         });
        }
    }
}

use gfx;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::Slice;
use cgmath::{Matrix4, SquareMatrix};
pub use ::target::{ColorFormat, GeometryBuffer};

gfx_vertex_struct!( Vertex {
    pos: [i32; 2] = "a_Pos",
    tex_coord: [i32; 2] = "a_TexCoord",
});


pub struct Clear;

impl<R> ::Pass<R> for Clear
    where R: gfx::Resources
{
    type Arg = ::pass::Clear;
    type Target = GeometryBuffer<R>;

    fn apply<C>(&self, c: &::pass::Clear, target: &GeometryBuffer<R>, _: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        encoder.clear(&target.normal, [0.; 4]);
        encoder.clear(&target.ka, c.color);
        encoder.clear(&target.kd, c.color);
        encoder.clear_depth(&target.depth, 1.0);
    }
}

pub static DRAW_VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    uniform mat4 u_Proj;
    uniform mat4 u_View;
    uniform mat4 u_Model;

    in vec3 a_Normal;
    in vec3 a_Pos;

    out vec3 v_Normal;

    void main() {
        v_Normal = mat3(u_Model) * a_Normal;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
";

pub static DRAW_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform vec4 u_Ka;
    uniform vec4 u_Kd;

    in vec3 v_Normal;

    out vec4 o_Ka;
    out vec4 o_Kd;
    out vec4 o_Normal;

    void main() {
        o_Ka = u_Ka;
        o_Kd = u_Kd;
        o_Normal = vec4(normalize(v_Normal), 0.);
    }
";

pub type GFormat = [f32; 4];
gfx_pipeline!( draw {
    vbuf: gfx::VertexBuffer<::VertexPosNormal> = (),
    ka: gfx::Global<[f32; 4]> = "u_Ka",
    kd: gfx::Global<[f32; 4]> = "u_Kd",
    model: gfx::Global<[[f32; 4]; 4]> = "u_Model",
    view: gfx::Global<[[f32; 4]; 4]> = "u_View",
    proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
    out_normal: gfx::RenderTarget<GFormat> = "o_Normal",
    out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Ka",
    out_kd: gfx::RenderTarget<gfx::format::Rgba8> = "o_Kd",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> =
        gfx::preset::depth::LESS_EQUAL_WRITE,
});

pub struct DrawPass<R: gfx::Resources>{
    pso: gfx::PipelineState<R, draw::Meta>
}

impl<R: gfx::Resources> DrawPass<R> {
    pub fn new<F>(factory: &mut F) -> DrawPass<R>
        where F: gfx::Factory<R>
    {
        DrawPass {
            pso: factory.create_pipeline_simple(
                DRAW_VERTEX_SRC,
                DRAW_FRAGMENT_SRC,
                draw::new()
            ).unwrap()
        }
    }
}

impl<R> ::Pass<R> for DrawPass<R>
    where R: gfx::Resources
{
    type Arg = ::pass::DrawNoShading;
    type Target = GeometryBuffer<R>;

    fn apply<C>(&self, arg: &::pass::DrawNoShading, target: &GeometryBuffer<R>, scenes: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        let scene = &scenes.scenes[&arg.scene];
        let camera = &scenes.cameras[&arg.camera];

        // every entity gets drawn
        for f in &scene.fragments {
            encoder.draw(
                &f.slice,
                &self.pso,
                &draw::Data {
                    ka: f.ka,
                    kd: f.kd,
                    model: f.transform,
                    view: camera.view,
                    proj: camera.projection,
                    vbuf: f.buffer.clone(),
                    out_normal: target.normal.clone(),
                    out_ka: target.ka.clone(),
                    out_kd: target.kd.clone(),
                    out_depth: target.depth.clone()
                }
            );
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


fn create_screen_fill_triangle<F, R>(factory: &mut F) -> (Buffer<R, Vertex>, Slice<R>)
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    let vertex_data = [
        Vertex { pos: [-3, -1], tex_coord: [-1, 0] },
        Vertex { pos: [ 1, -1], tex_coord: [1, 0] },
        Vertex { pos: [ 1,  3], tex_coord: [1, 2] },
    ];

    let buffer = factory.create_vertex_buffer(&vertex_data);
    let slice = Slice::new_match_vertex_buffer(&buffer);
    (buffer, slice)
}

pub struct BlitLayer<R: gfx::Resources> {
    buffer: Buffer<R, Vertex>,
    slice: Slice<R>,
    sampler: gfx::handle::Sampler<R>,
    pso: gfx::pso::PipelineState<R, blit::Meta>
}

impl<R> BlitLayer<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F) -> BlitLayer<R>
        where F: gfx::Factory<R>
    {
        let (buffer, slice) = create_screen_fill_triangle(factory);

        let sampler = factory.create_sampler(
            gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Scale,
                                       gfx::tex::WrapMode::Clamp)
        );

        BlitLayer{
            slice: slice,
            buffer: buffer,
            sampler: sampler,
            pso: factory.create_pipeline_simple(
                BLIT_VERTEX_SRC,
                BLIT_FRAGMENT_SRC,
                blit::new()
            ).unwrap()
        }
    }
}

impl<R> ::Pass<R> for BlitLayer<R>
    where R: gfx::Resources,
{
    type Arg = ::pass::BlitLayer;
    type Target = ::target::ColorBuffer<R>;

    fn apply<C>(&self, arg: &::pass::BlitLayer, target: &::target::ColorBuffer<R>, scenes: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        let src = &scenes.targets[&arg.gbuffer];
        let src = src.downcast_ref::<GeometryBuffer<R>>().unwrap();

        let layer = match arg.layer.as_ref() {
            "ka" => src.texture_ka.clone(),
            "kd" => src.texture_kd.clone(),
            "normal" => src.texture_normal.clone(),
            x => panic!("Unsupported layer {}", x)
        };

        encoder.draw(
            &self.slice,
            &self.pso,
            &blit::Data {
                vbuf: self.buffer.clone(),
                source: (layer, self.sampler.clone()),
                out: target.color.clone()
            }
        );
    }
}

pub static LIGHT_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core
    #define MAX_NUM_TOTAL_LIGHTS 128

    uniform mat4 u_Proj;
    uniform mat4 u_InvProj;
    uniform mat4 u_InvView;
    uniform vec4 u_Viewport;
    uniform int u_LightCount;

    struct Light {
        vec4 propagation;
        vec4 center;
        vec4 color;
    };

    layout (std140) uniform u_Lights {
        Light light[MAX_NUM_TOTAL_LIGHTS];
    };

    uniform sampler2D t_Kd;
    uniform sampler2D t_Depth;
    uniform sampler2D t_Normal;

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

        return u_InvView * u_InvProj * clip_pose;
    }

    void main() {
        float depth = texture(t_Depth, v_TexCoord).x;
        vec4 kd = texture(t_Kd, v_TexCoord);
        vec4 normal = texture(t_Normal, v_TexCoord);

        vec4 pos = calc_pos_from_window(vec3(gl_FragCoord.xy, depth));

        vec4 color = vec4(0., 0., 0., 0.);
        for (int i = 0; i < u_LightCount; i++) {
            vec4 delta = light[i].center - pos;
            float dist = length(delta);
            float inv_dist = 1. / dist;
            vec4 light_to_point_normal = delta * inv_dist;
            float intensity = dot(light[i].propagation.xyz, vec3(1., inv_dist, inv_dist * inv_dist));
            color += kd * light[i].color * intensity * max(0, dot(light_to_point_normal, normal));
        }
        o_Color = color;
    }
";


gfx_defines!(
    constant PointLight {
        propagation: [f32; 4] = "propagation",
        center: [f32; 4] = "center",
        color: [f32; 4] = "color",
    }

    pipeline light {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        kd: gfx::TextureSampler<[f32; 4]> = "t_Kd",
        normal: gfx::TextureSampler<[f32; 4]> = "t_Normal",
        depth: gfx::TextureSampler<f32> = "t_Depth",
        out: gfx::BlendTarget<ColorFormat> = ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ADD),

        light_count: gfx::Global<i32> = "u_LightCount",
        lights: gfx::ConstantBuffer<PointLight> = "u_Lights",
        viewport: gfx::Global<[f32; 4]> = "u_Viewport",
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
        inv_proj: gfx::Global<[[f32; 4]; 4]> = "u_InvProj",
        inv_view: gfx::Global<[[f32; 4]; 4]> = "u_InvView",
    }
);

pub struct LightingPass<R: gfx::Resources> {
    buffer: Buffer<R, Vertex>,
    lights: Buffer<R, PointLight>,
    slice: Slice<R>,
    sampler: gfx::handle::Sampler<R>,
    pso: gfx::pso::PipelineState<R, light::Meta>
}

fn pad(x: [f32; 3]) -> [f32; 4] {
    [x[0], x[1], x[2], 0.]
}

impl<R: gfx::Resources> LightingPass<R> {
    pub fn new<F>(factory: &mut F) -> LightingPass<R>
        where F: gfx::Factory<R>
    {
        let (buffer, slice) = create_screen_fill_triangle(factory);
        let sampler = factory.create_sampler(
            gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Scale,
                                       gfx::tex::WrapMode::Clamp)
        );

        let lights = factory.create_constant_buffer(128);
        LightingPass{
            lights: lights,
            buffer: buffer,
            slice: slice,
            sampler: sampler,
            pso: factory.create_pipeline_simple(
                BLIT_VERTEX_SRC,
                LIGHT_FRAGMENT_SRC,
                light::new()
            ).unwrap()
        }
    }
}

impl<R> ::Pass<R> for LightingPass<R>
    where R: gfx::Resources
{
    type Arg = ::pass::Lighting;
    type Target = ::target::ColorBuffer<R>;

    fn apply<C>(&self, arg: &::pass::Lighting, target: &::target::ColorBuffer<R>, scenes: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        let scene = &scenes.scenes[&arg.scene];
        let camera = &scenes.cameras[&arg.camera];
        let src = &scenes.targets[&arg.gbuffer];
        let src = src.downcast_ref::<GeometryBuffer<R>>().unwrap();

        let (w, h, _, _) = src.kd.get_dimensions();
        for lights in scene.lights.chunks(128) {
            let mut lights: Vec<_> = lights.iter().map(|l| PointLight{
                    propagation: [l.propagation_constant, l.propagation_linear, l.propagation_r_square, 0.],
                    color: l.color,
                    center: pad(l.center)
                }).collect();

            let count = lights.len();
            while lights.len() < 128 {
                lights.push(PointLight{
                    propagation: [0., 0., 0., 0.],
                    color: [0., 0., 0., 0.],
                    center: [0., 0., 0., 0.],
                })
            }

            encoder.update_buffer(&self.lights, &lights[..], 0).unwrap();
            encoder.draw(
                &self.slice,
                &self.pso,
                &light::Data {
                    vbuf: self.buffer.clone(),
                    kd: (src.texture_kd.clone(), self.sampler.clone()),
                    normal: (src.texture_normal.clone(), self.sampler.clone()),
                    depth: (src.texture_depth.clone(), self.sampler.clone()),
                    out: target.color.clone(),
                    inv_proj: Matrix4::from(camera.projection).invert().unwrap().into(),
                    inv_view: Matrix4::from(camera.view).invert().unwrap().into(),
                    proj: camera.projection,
                    viewport: [0., 0., w as f32, h as f32],
                    light_count: count as i32,
                    lights: self.lights.clone()
                }
            );

        }
    }
}

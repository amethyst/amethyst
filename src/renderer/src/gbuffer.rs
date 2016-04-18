
use gfx;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::Slice;
pub use ColorFormat;

gfx_vertex_struct!( Vertex {
    pos: [i32; 2] = "a_Pos",
    tex_coord: [i32; 2] = "a_TexCoord",
});

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


pub fn create_mesh<F, R>(factory: &mut F) -> (Buffer<R, Vertex>, Slice<R>)
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    let vertex_data = [
        Vertex { pos: [-3, -1], tex_coord: [-1, 0] },
        Vertex { pos: [ 1, -1], tex_coord: [1, 0] },
        Vertex { pos: [ 1,  3], tex_coord: [1, 2] },
    ];

    factory.create_vertex_buffer(&vertex_data)
}

pub type BlitPipeline<R> = gfx::pso::PipelineState<R, blit::Meta>;

pub fn create_blit_pipeline<F, R>(factory: &mut F) -> BlitPipeline<R>
    where R: gfx::Resources,
          F: gfx::Factory<R>
{
    factory.create_pipeline_simple(
        BLIT_VERTEX_SRC,
        BLIT_FRAGMENT_SRC,
        gfx::state::CullFace::Nothing,
        blit::new()
    ).unwrap()
}

pub static LIGHT_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform mat4 u_Proj;
    uniform mat4 u_InvProj;
    uniform mat4 u_InvView;
    uniform vec4 u_Viewport;
    uniform vec3 u_Propagation;
    uniform vec4 u_Center;
    uniform vec4 u_Color;

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
        vec4 delta = u_Center - pos;
        vec4 light_to_point_normal = normalize(delta);

        float dist = length(pos - u_Center);
        float intensity = dot(u_Propagation, vec3(1., 1./dist, 1/(dist*dist)));

        o_Color = kd * u_Color * intensity * max(0, dot(light_to_point_normal, normal));
    }
";

gfx_pipeline!( light {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    kd: gfx::TextureSampler<[f32; 4]> = "t_Kd",
    normal: gfx::TextureSampler<[f32; 4]> = "t_Normal",
    depth: gfx::TextureSampler<f32> = "t_Depth",
    out: gfx::BlendTarget<ColorFormat> =
        ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ADD),

    center: gfx::Global<[f32; 4]> = "u_Center",
    color: gfx::Global<[f32; 4]> = "u_Color",
    propagation: gfx::Global<[f32; 3]> = "u_Propagation",
    viewport: gfx::Global<[f32; 4]> = "u_Viewport",
    proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
    inv_proj: gfx::Global<[[f32; 4]; 4]> = "u_InvProj",
    inv_view: gfx::Global<[[f32; 4]; 4]> = "u_InvView",
});


pub type LightPipeline<R> = gfx::pso::PipelineState<R, light::Meta>;

pub fn create_light_pipline<F, R>(factory: &mut F) -> LightPipeline<R>
    where R: gfx::Resources,
          F: gfx::Factory<R>
{
    factory.create_pipeline_simple(
        BLIT_VERTEX_SRC,
        LIGHT_FRAGMENT_SRC,
        gfx::state::CullFace::Nothing,
        light::new()
    ).unwrap()
}

pub struct GBufferTarget<R: gfx::Resources> {
    pub normal: gfx::handle::RenderTargetView<R, [f32; 4]>,
    pub ka: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub kd: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub depth: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,
}

pub struct GBufferShaderResource<R: gfx::Resources> {
    pub normal: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub ka: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub kd: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub depth: gfx::handle::ShaderResourceView<R, f32>,
}

impl<R> GBufferTarget<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F, (width, height): (u16, u16)) -> (Self, GBufferShaderResource<R>)
        where F: gfx::Factory<R>
    {
        let (_, texture_normal,  normal) = factory.create_render_target(width, height).unwrap();
        let (_, texture_ka,  ka) = factory.create_render_target(width, height).unwrap();
        let (_, texture_kd,  kd) = factory.create_render_target(width, height).unwrap();
        let (_, texture_depth, depth) = factory.create_depth_stencil(width, height).unwrap();

        (
            GBufferTarget{
                normal: normal,
                kd: kd,
                ka: ka,
                depth: depth
            },
            GBufferShaderResource{
                normal: texture_normal,
                ka: texture_ka,
                kd: texture_kd,
                depth: texture_depth
            }
        )
    }
}


    /*pub fn render<C>(&mut self,
                     scene: &Scene<R, VertexPosNormal>,
                     encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {

        // clear the gbuffer
        encoder.clear(&self.gbuf_target.normal, [0.; 4]);
        encoder.clear(&self.gbuf_target.ka, [0.; 4]);
        encoder.clear(&self.gbuf_target.kd, [0.; 4]);
        encoder.clear_depth(&self.gbuf_target.depth, 1.0);
        encoder.clear_stencil(&output_depth, 0);

        // every entity gets drawn
        for e in &scene.entities {
            encoder.update_constant_buffer(&self.flat_uniform_vs,
                &forward::VertexUniforms {
                    view: scene.view,
                    proj: scene.projection,
                    model: e.transform
                }
            );
            encoder.update_constant_buffer(&self.flat_uniform_fs,
                &forward::FlatFragmentUniforms{
                    ka: e.ka,
                    kd: e.kd
                }
            );

            encoder.draw(
                &e.slice,
                &self.pipeline_foward,
                &forward::flat::Data {
                    vbuf: e.buffer.clone(),
                    uniform_vs: self.flat_uniform_vs.clone(),
                    uniform_fs: self.flat_uniform_fs.clone(),
                    out_normal: self.gbuf_target.normal.clone(),
                    out_ka: self.gbuf_target.ka.clone(),
                    out_kd: self.gbuf_target.kd.clone(),
                    out_depth: self.gbuf_target.depth.clone()
                }
            );
        }

        // blit the gbuffer to the screen
        encoder.draw(
            &self.blit_slice,
            &self.blit_pipeline,
            &gbuffer::blit::Data {
                vbuf: self.blit_mesh.clone(),
                source: (self.gbuf_texture.ka.clone(), self.blit_sampler.clone()),
                out: output.clone(),
            }
        );

        for l in &scene.lights {
            encoder.draw(
                &self.blit_slice,
                &self.light_pipeline,
                &gbuffer::light::Data {
                    vbuf: self.blit_mesh.clone(),
                    kd: (self.gbuf_texture.kd.clone(), self.blit_sampler.clone()),
                    normal: (self.gbuf_texture.normal.clone(), self.blit_sampler.clone()),
                    depth: (self.gbuf_texture.depth.clone(), self.blit_sampler.clone()),
                    out: output.clone(),
                    color: l.color,
                    center: [l.center[0], l.center[1], l.center[2], 1.],
                    propagation: [
                        l.propagation_constant,
                        l.propagation_linear,
                        l.propagation_r_square,
                    ],
                    inv_proj: Matrix4::from(scene.projection).invert().unwrap().into(),
                    inv_view: Matrix4::from(scene.view).invert().unwrap().into(),
                    proj: scene.projection,
                    viewport: [0., 0., 800., 600.]
                }
            );
        }
    }*/
use gfx;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::Slice;
pub use ::target::{ColorFormat, GeometryBuffer};

gfx_defines!(
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    constant FXAAArg {
        inverse_texture_size: [f32; 2] = "u_InverseTextureSize",
    }

    pipeline fxaa {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        source: gfx::TextureSampler<[f32; 4]> = "t_Source",
        out: gfx::RenderTarget<ColorFormat> = "o_Color",
        inverse_texture_size: gfx::ConstantBuffer<FXAAArg> = "Arg",
    }

    pipeline null {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        source: gfx::TextureSampler<[f32; 4]> = "t_Source",
        out: gfx::RenderTarget<ColorFormat> = "o_Color",
    }
);

pub static FXAA_VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    in vec2 a_Pos;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = vec4(a_Pos, 0.0, 1.0);
    }
";

pub static FXAA_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform sampler2D t_Source;
    layout (std140) uniform Arg {
        uniform vec2 u_InverseTextureSize;
    };

    in vec2 v_TexCoord;

    out vec4 o_Color;

    void main() {
        // o_Color = texture(t_Source, v_TexCoord);
        float FXAA_SPAN_MAX = 8.0;
        float FXAA_REDUCE_MIN = 1.0/128.0;
        float FXAA_REDUCE_MUL = 1.0/8.0;

        vec3 luma = vec3(0.2126, 0.7152, 0.0722);
        float lumaM = dot(texture(t_Source, v_TexCoord).xyz, luma);
        float lumaTL = dot(textureOffset(t_Source, v_TexCoord, ivec2(-1, -1)).xyz, luma);
        float lumaTR = dot(textureOffset(t_Source, v_TexCoord, ivec2( 1, -1)).xyz, luma);
        float lumaBR = dot(textureOffset(t_Source, v_TexCoord, ivec2( 1,  1)).xyz, luma);
        float lumaBL = dot(textureOffset(t_Source, v_TexCoord, ivec2(-1,  1)).xyz, luma);

        vec2 dir;
        dir.x = -((lumaTL + lumaTR) - (lumaBL + lumaBR));
        dir.y = ((lumaTL + lumaBL) - (lumaTR + lumaBR));

        float dirReduce = max((lumaTL + lumaTR + lumaBR + lumaBL) * (FXAA_REDUCE_MUL * 0.25), FXAA_REDUCE_MIN);
        float dirScaleFactor = 1.0/(min(abs(dir.x), abs(dir.y)) + dirReduce);
        dir = min(vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX), max(vec2(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX), dir * dirScaleFactor) * u_InverseTextureSize);

        vec3 result1 = (1/2.0) * (
                       texture(t_Source, v_TexCoord + dir * vec2(1.0/3.0 - 0.5)).xyz +
                       texture(t_Source, v_TexCoord + dir * vec2(2.0/3.0 - 0.5)).xyz);

        vec3 result2 = result1 * 1.0/2.0 + (1/4.0) * (
                       texture(t_Source, v_TexCoord + dir * vec2(0.0/3.0 - 0.5)).xyz +
                       texture(t_Source, v_TexCoord + dir * vec2(3.0/3.0 - 0.5)).xyz);

        float lumaMin = min(lumaM, min(min(lumaTL, lumaTR), min(lumaBR, lumaBL)));
        float lumaMax = max(lumaM, max(max(lumaTL, lumaTR), max(lumaBR, lumaBL)));
        float lumaResult2 = dot(luma, result2);

        if(lumaResult2 < lumaMin || lumaResult2 > lumaMax)
            o_Color = vec4(result1, 1.0);
        else
            o_Color = vec4(result2, 1.0);
    }
";

fn create_screen_fill_triangle<F, R>(factory: &mut F) -> (Buffer<R, Vertex>, Slice<R>)
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    let vertex_data = [
        Vertex { pos: [-3., -1.], tex_coord: [-1., 0.] },
        Vertex { pos: [ 1., -1.], tex_coord: [ 1., 0.] },
        Vertex { pos: [ 1.,  3.], tex_coord: [ 1., 2.] },
    ];

    let buffer = factory.create_vertex_buffer(&vertex_data);
    let slice = Slice::new_match_vertex_buffer(&buffer);
    (buffer, slice)
}

pub struct FXAA<R: gfx::Resources> {
    buffer: Buffer<R, Vertex>,
    slice: Slice<R>,
    sampler: gfx::handle::Sampler<R>,
    pso: gfx::pso::PipelineState<R, fxaa::Meta>,
    inverse_texture_size: Buffer<R, FXAAArg>,
}

impl<R> FXAA<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F) -> FXAA<R>
        where F: gfx::Factory<R>
    {
        let (buffer, slice) = create_screen_fill_triangle(factory);

        let sampler = factory.create_sampler(
            gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Scale,
                                       gfx::tex::WrapMode::Clamp)
        );

        let inverse_texture_size = factory.create_constant_buffer(1);

        FXAA {
            slice: slice,
            buffer: buffer,
            sampler: sampler,
            pso: factory.create_pipeline_simple(
                FXAA_VERTEX_SRC,
                FXAA_FRAGMENT_SRC,
                fxaa::new()
            ).unwrap(),
            inverse_texture_size: inverse_texture_size,
        }
    }
}

impl<R> ::Pass<R> for FXAA<R>
    where R: gfx::Resources,
{
    type Arg = ::pass::FXAA;
    type Target = ::target::ColorBuffer<R>;

    fn apply<C>(&self, arg: &::pass::FXAA, target: &::target::ColorBuffer<R>, scenes: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        let src = &scenes.targets[&arg.source];
        let src = src.downcast_ref::<::target::ColorBuffer<R>>().unwrap();

        let layer = src.texture_color.clone();
        let layer = layer.unwrap();

        encoder.update_constant_buffer(
            &self.inverse_texture_size,
            &FXAAArg {
                inverse_texture_size: arg.inverse_texture_size,
            },
        );

        encoder.draw(
            &self.slice,
            &self.pso,
            &fxaa::Data {
                vbuf: self.buffer.clone(),
                source: (layer, self.sampler.clone()),
                out: target.color.clone(),
                inverse_texture_size: self.inverse_texture_size.clone(),
            }
        );
    }
}

pub static NULL_VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    in vec2 a_Pos;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = vec4(a_Pos, 0.0, 1.0);
    }
";

pub static NULL_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform sampler2D t_Source;

    in vec2 v_TexCoord;
    out vec4 o_Color;

    void main() {
        o_Color = texture(t_Source, v_TexCoord);
    }
";

pub struct Null<R: gfx::Resources> {
    buffer: Buffer<R, Vertex>,
    slice: Slice<R>,
    sampler: gfx::handle::Sampler<R>,
    pso: gfx::pso::PipelineState<R, null::Meta>,
}

impl<R> Null<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F) -> Null<R>
        where F: gfx::Factory<R>
    {
        let (buffer, slice) = create_screen_fill_triangle(factory);

        let sampler = factory.create_sampler(
            gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Scale,
                                       gfx::tex::WrapMode::Clamp)
        );

        Null {
            slice: slice,
            buffer: buffer,
            sampler: sampler,
            pso: factory.create_pipeline_simple(
                NULL_VERTEX_SRC,
                NULL_FRAGMENT_SRC,
                null::new()
            ).unwrap(),
        }
    }
}

impl<R> ::Pass<R> for Null<R>
    where R: gfx::Resources,
{
    type Arg = ::pass::Null;
    type Target = ::target::ColorBuffer<R>;

    fn apply<C>(&self, arg: &::pass::Null, target: &::target::ColorBuffer<R>, scenes: &::Frame<R>, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        let src = &scenes.targets[&arg.source];
        let src = src.downcast_ref::<::target::ColorBuffer<R>>().unwrap();

        let layer = src.texture_color.clone();
        let layer = layer.unwrap();

        encoder.draw(
            &self.slice,
            &self.pso,
            &null::Data {
                vbuf: self.buffer.clone(),
                source: (layer, self.sampler.clone()),
                out: target.color.clone(),
            }
        );
    }
}

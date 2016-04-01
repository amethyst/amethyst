
use gfx;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::Slice;
pub use ColorFormat;

gfx_vertex_struct!( Vertex {
    pos: [i8; 2] = "a_Pos",
    tex_coord: [i8; 2] = "a_TexCoord",
});

gfx_pipeline!( blit {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    tex: gfx::TextureSampler<[f32; 4]> = "t_BlitTex",
    out: gfx::RenderTarget<ColorFormat> = "Target0",
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

    uniform sampler2D t_BlitTex;
    in vec2 v_TexCoord;
    out vec4 o_Color;

    void main() {
        vec4 tex = texture(t_BlitTex, v_TexCoord);
        o_Color = tex;
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
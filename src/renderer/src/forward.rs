
use gfx;
use gfx::traits::FactoryExt;
pub use VertexPosNormal;

pub static FORWARD_VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    uniform mat4 u_Proj;
    uniform mat4 u_View;
    uniform mat4 u_Model;

    in vec3 a_Pos;
    in vec3 a_Normal;

    void main() {
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
";

pub static FORWARD_FLAT_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform vec4 u_Ka;

    out vec4 o_Ka;

    void main() {
        o_Ka = u_Ka;
    }
";

pub type GFormat = [f32; 4];

gfx_constant_struct!( VertexUniforms {
    model: [[f32; 4]; 4] = "u_Model",
    view: [[f32; 4]; 4] = "u_View",
    proj: [[f32; 4]; 4] = "u_Proj",
});

gfx_constant_struct!( FlatFragmentUniforms {
    ka: [f32; 4] = "u_Ka",
    kd: [f32; 4] = "u_Kd",
});

gfx_pipeline!( flat {
    vbuf: gfx::VertexBuffer<VertexPosNormal> = (),
    ka: gfx::Global<[f32; 4]> = "u_Ka",
    model: gfx::Global<[[f32; 4]; 4]> = "u_Model",
    view: gfx::Global<[[f32; 4]; 4]> = "u_View",
    proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
    out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Ka",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
});

pub type FlatPipeline<R> = gfx::pso::PipelineState<R, flat::Meta>;

pub fn create_flat_pipeline<F, R>(factory: &mut F) -> FlatPipeline<R>
    where R: gfx::Resources,
          F: gfx::Factory<R>
{
    factory.create_pipeline_simple(
        FORWARD_VERTEX_SRC,
        FORWARD_FLAT_FRAGMENT_SRC,
        gfx::state::CullFace::Back,
        flat::new()
    ).unwrap()
}
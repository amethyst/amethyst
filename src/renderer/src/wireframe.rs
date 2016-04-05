
use gfx;
use gfx::traits::FactoryExt;
pub use forward::VertexPosNormal;

pub static VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    uniform mat4 u_Model;
    uniform mat4 u_View;
    uniform mat4 u_Proj;

    in vec3 a_Pos;

    void main() {
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
";

pub static GEOMETRY_SRC: &'static [u8] = b"
    #version 150 core

    layout(triangles) in;
    layout(line_strip, max_vertices=4) out;

    void main() {
        gl_Position = gl_in[0].gl_Position;
        EmitVertex();
        gl_Position = gl_in[1].gl_Position;
        EmitVertex();
        gl_Position = gl_in[2].gl_Position;
        EmitVertex();
        gl_Position = gl_in[0].gl_Position;
        EmitVertex();
        EndPrimitive();
    }
";

pub static FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform vec4 u_Ka;
    out vec4 o_Ka;

    void main() {
        o_Ka = u_Ka;
    }
";

pub type GFormat = [f32; 4];

gfx_pipeline!( wireframe {
    vbuf: gfx::VertexBuffer<VertexPosNormal> = (),
    ka: gfx::Global<[f32; 4]> = "u_Ka",
    model: gfx::Global<[[f32; 4]; 4]> = "u_Model",
    view: gfx::Global<[[f32; 4]; 4]> = "u_View",
    proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
    out_ka: gfx::RenderTarget<gfx::format::Rgba8> = "o_Ka",
});

pub type WireframePipeline<R> = gfx::pso::PipelineState<R, wireframe::Meta>;

pub fn create_wireframe_pipeline<F, R>(factory: &mut F) -> WireframePipeline<R>
    where R: gfx::Resources,
          F: gfx::Factory<R>
{
    let vs = factory.create_shader_vertex(VERTEX_SRC).unwrap();
    let gs = factory.create_shader_geometry(GEOMETRY_SRC).unwrap();
    let fs = factory.create_shader_pixel(FRAGMENT_SRC).unwrap();

    factory.create_pipeline_state(
        &gfx::ShaderSet::Geometry(vs, gs, fs),
        gfx::Primitive::TriangleList,
        gfx::state::Rasterizer::new_fill(gfx::state::CullFace::Nothing),
        wireframe::new()
    ).unwrap()
}
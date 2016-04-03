
use gfx;
use gfx::traits::FactoryExt;

pub static FORWARD_VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    layout(std140)
    uniform FowardVertexUniforms {
        mat4 u_Model;
        mat4 u_View;
        mat4 u_Proj;
    };

    in vec3 a_Pos;
    in vec3 a_Normal;

    out vec3 v_FragPos;
    out vec3 v_Normal;

    void main() {
        v_FragPos = (u_Model * vec4(a_Pos, 1.0)).xyz;
        v_Normal = mat3(u_Model) * a_Normal;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
";

pub static FORWARD_FLAT_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    uniform FlatFragmentUniforms {
        vec4 u_Ka;
        vec4 u_Kd;
    };

    in vec3 v_FragPos;
    in vec3 v_Normal;

    out vec4 o_Normal;
    out vec4 o_Ka;
    out vec4 o_Kd;

    void main() {
        vec3 n = normalize(v_Normal);

        o_Normal = vec4(n, 0.0);
        o_Ka = u_Ka;
        o_Kd = u_Kd;
    }
";

pub type GFormat = [f32; 4];

// placeholder
gfx_vertex_struct!( VertexPosNormal {
    pos: [f32; 3] = "a_Pos",
    normal: [f32; 3] = "a_Normal",
});

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
    uniform_vs: gfx::ConstantBuffer<VertexUniforms> = "FowardVertexUniforms",
    uniform_fs: gfx::ConstantBuffer<FlatFragmentUniforms> = "FlatFragmentUniforms",

    out_normal: gfx::RenderTarget<GFormat> = "o_Normal",
    out_ka: gfx::RenderTarget<GFormat> = "o_Ka",
    out_kd: gfx::RenderTarget<GFormat> = "o_Kd",
    out_depth: gfx::DepthTarget<gfx::format::Depth> =
        gfx::preset::depth::LESS_EQUAL_WRITE,
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
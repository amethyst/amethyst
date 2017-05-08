//! Blits a color or depth buffer from one Target onto another.

use gfx;
use pipe::pass::PassBuilder;
use types::{Buffer, ColorFormat, Factory, Slice};
use vertex::PosTex;

static VERT_SRC: &'static [u8] = b"
    #version 150 core
    in ivec3 a_position;
    in ivec2 a_tex_coord;
    out vec2 v_tex_coord;
    void main() {
        v_tex_coord = a_tex_coord;
        gl_Position = vec4(a_position, 1.0);
    }
";

static FRAG_SRC: &'static [u8] = b"
    #version 150 core
    uniform sampler2D t_source;
    in vec2 v_tex_coord;
    out vec4 o_color;
    void main() {
        o_Color = texture(t_source, v_tex_coord);
    }
";

gfx_pipeline!(
    blit {
        vbuf: gfx::VertexBuffer<PosTex> = (),
        source: gfx::TextureSampler<[f32; 4]> = "t_source",
        out: gfx::RenderTarget<ColorFormat> = "o_color",
    }
);

/// Blits a color or depth buffer from one Target onto another.
#[derive(Clone, Debug, PartialEq)]
pub struct BlitBuffer {
    buf_idx: Option<usize>,
    target: String,
}

impl BlitBuffer {
    /// Blits the color buffer of the given target onto the Stage's target.
    pub fn color_buf<T: Into<String>>(target_name: T, i: usize) -> BlitBuffer {
        BlitBuffer {
            buf_idx: Some(i),
            target: target_name.into(),
        }
    }

    /// Blits the depth buffer of the given target onto the Stage's target.
    pub fn depth_buf<T: Into<String>>(target_name: T) -> BlitBuffer {
        BlitBuffer {
            buf_idx: None,
            target: target_name.into(),
        }
    }
}

impl Into<PassBuilder> for BlitBuffer {
    fn into(self) -> PassBuilder {
        use gfx::texture::{FilterMethod, WrapMode};

        let effect = Effect::new()
            .with_sampler("blit", FilterMethod::Scale, WrapMode::Clamp)
            .with_input_target(self.target, "blit")
            .with_simple_prog(VERT_SRC, FRAG_SRC);

        PassBuilder::postproc(effect, move |ref mut enc, ref out, ref effect| {
                let buf = if let Some(i) = buf_idx {
                    data.targets[0].color_buf(i).unwrap().target_view
                } else {
                    data.targets[0].depth_buf().unwrap().target_view
                };

                enc.draw(&slice, &data.pso.unwrap(), &blit::Data {
                    vbuf: vbuf,
                    source: (buf, data.samplers[0].clone()),
                    out: out.color_buf(0).unwrap().target_view.clone(),
                });
            })
    }
}

fn create_fullscreen_quad(fac: &mut Factory) -> (Buffer<PosTex>, Slice) {
    use gfx::traits::FactoryExt;

    let verts = [PosTex {
                     a_position: [-3.0, -1.0, 0.0],
                     a_tex_coord: [-1.0, 0.0],
                 },
                 PosTex {
                     a_position: [1.0, -1.0, 0.0],
                     a_tex_coord: [1.0, 0.0],
                 },
                 PosTex {
                     a_position: [1.0, 3.0, 0.0],
                     a_tex_coord: [1.0, 2.0],
                 }];

    fac.create_vertex_buffer_with_slice(&verts, ())
}

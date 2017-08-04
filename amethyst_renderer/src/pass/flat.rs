//! Simple flat forward drawing pass.

use error::Result;
use cgmath::{Matrix4, One};
use gfx::pso::buffer::ElemStride;
use pipe::pass::Pass;
use pipe::{DepthMode, Effect, NewEffect};
use scene::{Model, Scene};
use std::marker::PhantomData;
use types::Encoder;
use vertex::{Attribute, Position, TextureCoord, VertexFormat, WithField};

static VERT_SRC: &[u8] = include_bytes!("shaders/vertex/basic.glsl");
static FRAG_SRC: &[u8] = include_bytes!("shaders/fragment/flat.glsl");

/// Draw mesh without lighting
#[derive(Clone, Debug, PartialEq)]
pub struct DrawFlat<V> {
    vertex_attributes: [(&'static str, Attribute); 2],
    _pd: PhantomData<V>,
}

impl<V> DrawFlat<V>
    where V: VertexFormat + WithField<Position> + WithField<TextureCoord>
{
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        DrawFlat {
            vertex_attributes: [("position", V::attribute::<Position>()),
                                ("tex_coord", V::attribute::<TextureCoord>())],
            _pd: PhantomData,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct VertexArgs {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

impl<V: VertexFormat> Pass for DrawFlat<V> {
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        effect.simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_vertex_buffer(self.vertex_attributes.as_ref(), V::size() as ElemStride, 0)
            .with_texture("albedo")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply(&self, enc: &mut Encoder, effect: &mut Effect, scene: &Scene, model: &Model) {
        let vertex_args = scene
             .active_camera()
             .map(|cam| {
                      VertexArgs {
                          proj: cam.proj.into(),
                          view: cam.to_view_matrix().into(),
                          model: model.pos.into(),
                      }
                  })
             .unwrap_or_else(|| {
                                 VertexArgs {
                                     proj: Matrix4::one().into(),
                                     view: Matrix4::one().into(),
                                     model: model.pos.into(),
                                 }
                             });

         effect.update_constant_buffer("VertexArgs", &vertex_args, enc);
         effect.data.textures.push(model.material.albedo.view().clone());
         effect.data.samplers.push(model.material.albedo.sampler().clone());

         effect.draw(model, enc);         
    }
}

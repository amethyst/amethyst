//! Blits a color or depth buffer from one Target onto another.

use cam::Camera;
use cgmath::{Matrix4, One};
use gfx;
use gfx::pso::buffer::{ElemStride, NonInstanced};
use pipe::pass::PassBuilder;
use pipe::{DepthMode, Effect};
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::mem::{self, transmute};
use vertex::{Attribute, Color, Position, TextureCoord, VertexFormat, WithField};

static VERT_SRC: &'static [u8] = include_bytes!("shaders/vertex/basic.glsl");
static FRAG_SRC: &'static [u8] = include_bytes!("shaders/fragment/flat.glsl");

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

static SAMPLER_NAMES: [&'static str; 1] = ["albedo"];

impl<'a, V> Into<PassBuilder<'a>> for &'a DrawFlat<V>
    where V: VertexFormat
{
    fn into(self) -> PassBuilder<'a> {
        use gfx::texture::{FilterMethod, WrapMode};

        #[derive(Clone, Copy, Debug)]
        struct VertexArgs {
            proj: [[f32; 4]; 4],
            view: [[f32; 4]; 4],
            model: [[f32; 4]; 4],
        };

        let effect = Effect::new_simple_prog(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_vertex_buffer(self.vertex_attributes.as_ref(), V::size() as ElemStride, 0)
            .with_sampler(&SAMPLER_NAMES, FilterMethod::Scale, WrapMode::Clamp)
            .with_texture("albedo")
            .with_output("color", Some(DepthMode::LessEqualWrite));

        PassBuilder::model(effect,
                           move |ref mut enc, ref out, ref effect, ref scene, ref model| {
            let vertex_args = scene
                .active_camera()
                .map(|cam| {
                         VertexArgs {
                             proj: cam.proj.into(),
                             view: Matrix4::look_at(cam.eye, cam.eye + cam.forward, cam.up).into(),
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
            let vertex_args_buf = effect.const_bufs["VertexArgs"].clone();

            // FIXME: update raw buffer without transmute
            enc.update_constant_buffer::<VertexArgs>(unsafe { transmute(&vertex_args_buf) },
                                                     &vertex_args);

            let mut data = effect.pso_data.clone();
            data.const_bufs.push(vertex_args_buf);
            let (vertex, slice) = model.mesh.geometry();
            data.vertex_bufs.push(vertex.clone());
            data.samplers.push(effect.samplers["albedo"].clone());
            data.textures.push(model.material.albedo.view().clone());
            data.out_colors
                .extend(out.color_buf(0).map(|cb| cb.as_output.clone()));
            data.out_depth = out.depth_buf().map(|db| (db.as_output.clone(), (0, 0)));
            enc.draw(slice, &effect.pso, &data);
        })
    }
}

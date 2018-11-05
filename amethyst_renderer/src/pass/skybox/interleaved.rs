//! Skybox pass

use amethyst_core::{
    cgmath::{Matrix4, One},
    specs::{Read, ReadStorage},
    transform::GlobalTransform,
};
use {
    error::Result,
    get_camera,
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    pass::util::set_attribute_buffers,
    set_vertex_args, ActiveCamera, Camera, Encoder, Factory, Mesh, Normal,
    PosNormTex, Position, Query, Shape, TexCoord, VertexFormat,
};
use gfx::pso::buffer::ElemStride;
use glsl_layout::{mat4, Uniform};
use std::marker::PhantomData;

use super::{VERT_SRC, FRAG_SRC};

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct VertexArgs {
    proj: mat4,
    view: mat4,
    model: mat4,
}

/// Draw a simple gradient skybox
///
/// # Type Parameters:
///
/// * `V`: `VertexFormat`
#[derive(Clone, Debug)]
pub struct DrawSkybox<V> {
    _marker: PhantomData<V>,
    mesh: Option<Mesh>,
}

impl<V> DrawSkybox<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    /// Create instance of `DrawSkybox` pass
    pub fn new() -> Self {
        DrawSkybox {
            _marker: PhantomData {},
            mesh: None,
        }
    }
}

impl<'a, V> PassData<'a> for DrawSkybox<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    type Data = (
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, GlobalTransform>,
    );
}

impl<V> Pass for DrawSkybox<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    fn compile(&mut self, mut effect: NewEffect) -> Result<Effect> {
        let verts = Shape::Cube.generate_vertices::<Vec<PosNormTex>>(None);
        self.mesh = Some(Mesh::build(verts).build(&mut effect.factory)?);

        effect
            .simple(VERT_SRC, FRAG_SRC)
            .without_back_face_culling()
            .with_raw_constant_buffer(
                "VertexArgs",
                std::mem::size_of::<<VertexArgs as Uniform>::Std140>(),
                1
            )
            .with_raw_vertex_buffer(
                PosNormTex::ATTRIBUTES, PosNormTex::size() as ElemStride, 0
            )
            .with_raw_global("camera_position")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        mut _factory: Factory,
        (active, camera, global): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        let mesh = self
            .mesh
            .as_ref()
            .expect("Pass doesn't seem to be compiled.");

        set_vertex_args(effect, encoder, camera, &GlobalTransform(Matrix4::one()));

        if !set_attribute_buffers(effect, &mesh, &[V::QUERIED_ATTRIBUTES]) {
            effect.clear();
            return;
        }

        effect.draw(mesh.slice(), encoder);
        effect.clear();
    }
}

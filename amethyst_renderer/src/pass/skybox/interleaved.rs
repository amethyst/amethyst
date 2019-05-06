//! Skybox pass

use std::marker::PhantomData;

use amethyst_core::{
    alga::general::SubsetOf,
    ecs::{Read, ReadStorage},
    math::{self as na, RealField},
    transform::Transform,
};
use amethyst_error::Error;

use crate::{
    get_camera,
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    set_vertex_args, ActiveCamera, Camera, Encoder, Factory, Mesh, PosTex, Rgba, Shape,
    VertexFormat,
};

use gfx::pso::buffer::ElemStride;
use glsl_layout::{mat4, vec4, Uniform};

use super::{SkyboxColor, FRAG_SRC, VERT_SRC};

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Uniform)]
pub(crate) struct VertexArgs {
    proj: mat4,
    view: mat4,
    model: mat4,
    rgba: vec4,
}

/// Draw a simple gradient skybox
///
/// # Type Parameters:
///
/// * `N`: `RealBound` (f32, f64)
#[derive(Clone, Debug)]
pub struct DrawSkybox<N> {
    mesh: Option<Mesh>,
    _pd: PhantomData<N>,
}

impl<N> DrawSkybox<N> {
    /// Create instance of `DrawSkybox` pass
    pub fn new() -> Self {
        DrawSkybox {
            mesh: None,
            _pd: PhantomData,
        }
    }
}

impl<'a, N> PassData<'a> for DrawSkybox<N>
where
    N: RealField,
{
    type Data = (
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transform<N>>,
        Read<'a, SkyboxColor>,
    );
}

impl<N> Pass for DrawSkybox<N>
where
    N: RealField + SubsetOf<f32>,
{
    fn compile(&mut self, mut effect: NewEffect<'_>) -> Result<Effect, Error> {
        let verts = Shape::Cube.generate_vertices::<Vec<PosTex>>(None);
        self.mesh = Some(Mesh::build(verts).build(&mut effect.factory)?);

        effect
            .simple(VERT_SRC, FRAG_SRC)
            .without_back_face_culling()
            .with_raw_constant_buffer(
                "VertexArgs",
                std::mem::size_of::<<VertexArgs as Uniform>::Std140>(),
                1,
            )
            .with_raw_vertex_buffer(PosTex::ATTRIBUTES, PosTex::size() as ElemStride, 0)
            .with_raw_global("camera_position")
            .with_raw_global("zenith_color")
            .with_raw_global("nadir_color")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        mut _factory: Factory,
        (active, camera, transform, skybox_color): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &transform);

        let mesh = self
            .mesh
            .as_ref()
            .expect("Pass doesn't seem to be compiled.");

        set_vertex_args(effect, encoder, camera, &na::one(), Rgba::WHITE);

        if let Some(vbuf) = mesh.buffer(PosTex::ATTRIBUTES) {
            effect.data.vertex_bufs.push(vbuf.clone());
        } else {
            effect.clear();
            return;
        }

        effect.update_global("zenith_color", Into::<[f32; 3]>::into(skybox_color.zenith));
        effect.update_global("nadir_color", Into::<[f32; 3]>::into(skybox_color.nadir));
        effect.draw(mesh.slice(), encoder);
        effect.clear();
    }
}

//! Skybox pass

use amethyst_core::{
    nalgebra as na,
    specs::{Read, ReadStorage},
    transform::GlobalTransform,
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
#[derive(Clone, Debug)]
pub struct DrawSkybox {
    mesh: Option<Mesh>,
}

impl DrawSkybox {
    /// Create instance of `DrawSkybox` pass
    pub fn new() -> Self {
        DrawSkybox { mesh: None }
    }
}

impl<'a> PassData<'a> for DrawSkybox {
    type Data = (
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, GlobalTransform>,
        Read<'a, SkyboxColor>,
    );
}

impl Pass for DrawSkybox {
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
        (active, camera, global, skybox_color): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        let mesh = self
            .mesh
            .as_ref()
            .expect("Pass doesn't seem to be compiled.");

        set_vertex_args(
            effect,
            encoder,
            camera,
            &GlobalTransform(na::one()),
            Rgba::WHITE,
        );

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

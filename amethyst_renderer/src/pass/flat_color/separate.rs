//! Simple flat forward drawing pass.

use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    specs::prelude::{Join, Read, ReadExpect, ReadStorage},
    transform::GlobalTransform,
};

use crate::{
    cam::{ActiveCamera, Camera},
    error::Result,
    hidden::{Hidden, HiddenPropagate},
    mesh::{Mesh, MeshHandle},
    mtl::{Material, MaterialDefaults},
    pass::skinning::{create_skinning_effect, setup_skinning_buffers},
    pass::util::{draw_mesh, get_camera, VertexArgs},
    pipe::pass::{Pass, PassData},
    pipe::{DepthMode, Effect, NewEffect},
    skinning::JointTransforms,
    tex::Texture,
    types::{Encoder, Factory},
    vertex::{Attributes, Color, Position, Separate, VertexFormat},
    visibility::Visibility,
};

use super::*;

static ATTRIBUTES: [Attributes<'static>; 2] = [
    Separate::<Position>::ATTRIBUTES,
    Separate::<Color>::ATTRIBUTES,
];

/// Draw mesh without lighting
///
/// See the [crate level documentation](index.html) for information about interleaved and separate
/// passes.
///
/// # Type Parameters
///
/// * `V`: `VertexFormat`
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawFlatColorSeparate {
    skinning: bool,
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl DrawFlatColorSeparate
where
    Self: Pass,
{
    /// Create instance of `DrawFlatColor` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable vertex skinning
    pub fn with_vertex_skinning(mut self) -> Self {
        self.skinning = true;
        self
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }
}

impl<'a> PassData<'a> for DrawFlatColorSeparate {
    type Data = (
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
        ReadExpect<'a, MaterialDefaults>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, JointTransforms>,
    );
}

impl Pass for DrawFlatColorSeparate {
    fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect> {
        use std::mem;
        let mut builder = if self.skinning {
            create_skinning_effect(effect, FRAG_SRC)
        } else {
            effect.simple(VERT_SRC, FRAG_SRC)
        };
        builder
            .with_raw_vertex_buffer(
                Separate::<Position>::ATTRIBUTES,
                Separate::<Position>::size() as ElemStride,
                0,
            )
            .with_raw_vertex_buffer(
                Separate::<Color>::ATTRIBUTES,
                Separate::<Color>::size() as ElemStride,
                0,
            );
        if self.skinning {
            setup_skinning_buffers(&mut builder);
        }
        builder.with_raw_constant_buffer(
            "VertexArgs",
            mem::size_of::<<VertexArgs as Uniform>::Std140>(),
            1,
        );
        match self.transparency {
            Some((mask, blend, depth)) => builder.with_blended_output("color", mask, blend, depth),
            None => builder.with_output("color", Some(DepthMode::LessEqualWrite)),
        };
        builder.build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (
            active,
            camera,
            mesh_storage,
            tex_storage,
            material_defaults,
            visibility,
            hidden,
            hidden_prop,
            mesh,
            material,
            global,
            joints,
            rgba,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        match visibility {
            None => {
                for (joint, mesh, material, global, rgba, _, _) in (
                    joints.maybe(),
                    &mesh,
                    &material,
                    &global,
                    rgba.maybe(),
                    !&hidden,
                    !&hidden_prop,
                )
                    .join()
                {
                    draw_mesh(
                        encoder,
                        effect,
                        self.skinning,
                        mesh_storage.get(mesh),
                        joint,
                        &tex_storage,
                        Some(material),
                        &material_defaults,
                        rgba,
                        camera,
                        Some(global),
                        &ATTRIBUTES,
                        &[],
                    );
                }
            }
            Some(ref visibility) => {
                for (joint, mesh, material, global, rgba, _) in (
                    joints.maybe(),
                    &mesh,
                    &material,
                    &global,
                    rgba.maybe(),
                    &visibility.visible_unordered,
                )
                    .join()
                {
                    draw_mesh(
                        encoder,
                        effect,
                        self.skinning,
                        mesh_storage.get(mesh),
                        joint,
                        &tex_storage,
                        Some(material),
                        &material_defaults,
                        rbga,
                        camera,
                        Some(global),
                        &ATTRIBUTES,
                        &[],
                    );
                }

                for entity in &visibility.visible_ordered {
                    if let Some(mesh) = mesh.get(*entity) {
                        draw_mesh(
                            encoder,
                            effect,
                            self.skinning,
                            mesh_storage.get(mesh),
                            joints.get(*entity),
                            &tex_storage,
                            material.get(*entity),
                            &material_defaults,
                            rgba,
                            camera,
                            global.get(*entity),
                            &ATTRIBUTES,
                            &[],
                        );
                    }
                }
            }
        }
    }
}

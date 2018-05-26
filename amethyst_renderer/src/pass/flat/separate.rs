//! Simple flat forward drawing pass.

use amethyst_assets::AssetStorage;
use amethyst_core::specs::prelude::{Entities, Join, Read, ReadExpect, ReadStorage};
use amethyst_core::transform::GlobalTransform;
use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pass::skinning::{create_skinning_effect, setup_skinning_buffers};
use pass::util::{draw_mesh, get_camera, setup_textures, VertexArgs};
use pipe::pass::{Pass, PassData};
use pipe::{DepthMode, Effect, NewEffect};
use skinning::JointTransforms;
use tex::Texture;
use types::{Encoder, Factory};
use vertex::{Attributes, Position, Separate, TexCoord, VertexFormat};
use visibility::Visibility;

static ATTRIBUTES: [Attributes<'static>; 2] = [
    Separate::<Position>::ATTRIBUTES,
    Separate::<TexCoord>::ATTRIBUTES,
];

/// Draw mesh without lighting
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawFlatSeparate {
    skinning: bool,
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl DrawFlatSeparate
where
    Self: Pass,
{
    /// Create instance of `DrawFlat` pass
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

impl<'a> PassData<'a> for DrawFlatSeparate {
    type Data = (
        Entities<'a>,
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
        ReadExpect<'a, MaterialDefaults>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, JointTransforms>,
    );
}

impl Pass for DrawFlatSeparate {
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
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
                Separate::<TexCoord>::ATTRIBUTES,
                Separate::<TexCoord>::size() as ElemStride,
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
        setup_textures(&mut builder, &TEXTURES);
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
            entities,
            active,
            camera,
            mesh_storage,
            tex_storage,
            material_defaults,
            visibility,
            mesh,
            material,
            global,
            joints,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        match visibility {
            None => for (entity, mesh, material, global) in
                (&*entities, &mesh, &material, &global).join()
            {
                draw_mesh(
                    encoder,
                    effect,
                    self.skinning,
                    mesh_storage.get(mesh),
                    joints.get(entity),
                    &tex_storage,
                    Some(material),
                    &material_defaults,
                    camera,
                    Some(global),
                    &ATTRIBUTES,
                    &TEXTURES,
                );
            },
            Some(ref visibility) => {
                for (entity, mesh, material, global, _) in (
                    &*entities,
                    &mesh,
                    &material,
                    &global,
                    &visibility.visible_unordered,
                ).join()
                {
                    draw_mesh(
                        encoder,
                        effect,
                        self.skinning,
                        mesh_storage.get(mesh),
                        joints.get(entity),
                        &tex_storage,
                        Some(material),
                        &material_defaults,
                        camera,
                        Some(global),
                        &ATTRIBUTES,
                        &TEXTURES,
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
                            camera,
                            global.get(*entity),
                            &ATTRIBUTES,
                            &TEXTURES,
                        );
                    }
                }
            }
        }
    }
}

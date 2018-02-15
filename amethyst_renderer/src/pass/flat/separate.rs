//! Simple flat forward drawing pass.

use amethyst_assets::AssetStorage;
use amethyst_core::transform::GlobalTransform;
use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use specs::{Entities, Fetch, Join, ReadStorage};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pass::skinning::{create_skinning_effect, setup_skinning_buffers};
use pass::util::{draw_mesh, get_camera, VertexArgs};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassData};
use skinning::JointTransforms;
use tex::Texture;
use transparent::{Transparent, TransparentBackToFront};
use types::{Encoder, Factory};
use vertex::{Attributes, Position, Separate, TexCoord, VertexFormat};

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
        Option<Fetch<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        Fetch<'a, TransparentBackToFront>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, JointTransforms>,
        ReadStorage<'a, Transparent>,
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
        builder
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_texture("albedo");
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
            back_to_front,
            mesh,
            material,
            global,
            joints,
            transparent,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        for (entity, mesh, material, global, _) in
            (&*entities, &mesh, &material, &global, !&transparent).join()
        {
            draw_mesh(
                encoder,
                effect,
                self.skinning,
                mesh_storage.get(mesh),
                joints.get(entity),
                &*tex_storage,
                Some(material),
                &*material_defaults,
                camera,
                Some(global),
                &ATTRIBUTES,
                &TEXTURES,
            );
        }

        for entity in &back_to_front.entities {
            if let Some(mesh) = mesh.get(*entity) {
                draw_mesh(
                    encoder,
                    effect,
                    self.skinning,
                    mesh_storage.get(mesh),
                    joints.get(*entity),
                    &*tex_storage,
                    material.get(*entity),
                    &*material_defaults,
                    camera,
                    global.get(*entity),
                    &ATTRIBUTES,
                    &TEXTURES,
                );
            }
        }
    }
}

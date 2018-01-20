//! Simple flat forward drawing pass.

use amethyst_assets::AssetStorage;
use amethyst_core::transform::Transform;
use gfx::pso::buffer::ElemStride;
use specs::{Entities, Fetch, Join, ReadStorage};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pass::skinning::{create_skinning_effect, set_skinning_buffers, setup_skinning_buffers};
use pass::util::{add_textures, set_attribute_buffers, set_vertex_args, VertexArgs};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassData};
use skinning::JointTransforms;
use tex::Texture;
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
}

impl<'a> PassData<'a> for DrawFlatSeparate {
    type Data = (
        Entities<'a>,
        Option<Fetch<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, JointTransforms>,
    );
}

impl Pass for DrawFlatSeparate {
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
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
            .with_texture("albedo")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
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
            mesh,
            material,
            global,
            joints,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera: Option<(&Camera, &Transform)> = active
            .and_then(|a| {
                let cam = camera.get(a.entity);
                let transform = global.get(a.entity);
                cam.into_iter().zip(transform.into_iter()).next()
            })
            .or_else(|| (&camera, &global).join().next());

        'drawable: for (entity, mesh, material, global) in
            (&*entities, &mesh, &material, &global).join()
        {
            let mesh = match mesh_storage.get(mesh) {
                Some(mesh) => mesh,
                None => continue,
            };

            if !set_attribute_buffers(effect, mesh, &ATTRIBUTES)
                || (self.skinning && !set_skinning_buffers(effect, mesh))
            {
                effect.clear();
                continue 'drawable;
            }

            set_vertex_args(effect, encoder, camera, global);

            if self.skinning {
                if let Some(joint) = joints.get(entity) {
                    effect.update_buffer("JointTransforms", &joint.matrices[..], encoder);
                }
            }

            add_textures(
                effect,
                &tex_storage,
                material,
                &material_defaults.0,
                &TEXTURES,
            );

            effect.draw(mesh.slice(), encoder);
            effect.clear();
        }
    }
}

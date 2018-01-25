//! Forward physically-based drawing pass.

use amethyst_assets::AssetStorage;
use amethyst_core::transform::Transform;
use gfx::pso::buffer::ElemStride;
use specs::{Entities, Fetch, Join, ReadStorage};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use light::Light;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pass::shaded_util::{set_light_args, setup_light_buffers};
use pass::skinning::{create_skinning_effect, set_skinning_buffers, setup_skinning_buffers};
use pass::util::{add_textures, set_attribute_buffers, set_vertex_args, setup_textures,
                 setup_vertex_args};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassData};
use resources::AmbientColor;
use skinning::JointTransforms;
use tex::Texture;
use types::{Encoder, Factory};
use vertex::{Attributes, Normal, Position, Separate, Tangent, TexCoord, VertexFormat};

static ATTRIBUTES: [Attributes<'static>; 4] = [
    Separate::<Position>::ATTRIBUTES,
    Separate::<Normal>::ATTRIBUTES,
    Separate::<Tangent>::ATTRIBUTES,
    Separate::<TexCoord>::ATTRIBUTES,
];

/// Draw mesh with physically based lighting
#[derive(Default, Clone, Debug, PartialEq)]
pub struct DrawPbmSeparate {
    skinning: bool,
}

impl DrawPbmSeparate {
    /// Create instance of `DrawPbm` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable vertex skinning
    pub fn with_vertex_skinning(mut self) -> Self {
        self.skinning = true;
        self
    }
}

impl<'a> PassData<'a> for DrawPbmSeparate {
    type Data = (
        Entities<'a>,
        Option<Fetch<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Fetch<'a, AmbientColor>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Light>,
        ReadStorage<'a, JointTransforms>,
    );
}

impl Pass for DrawPbmSeparate {
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
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
                Separate::<Normal>::ATTRIBUTES,
                Separate::<Normal>::size() as ElemStride,
                0,
            )
            .with_raw_vertex_buffer(
                Separate::<Tangent>::ATTRIBUTES,
                Separate::<Tangent>::size() as ElemStride,
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
        setup_vertex_args(&mut builder);
        setup_light_buffers(&mut builder);
        setup_textures(&mut builder, &TEXTURES);
        builder
            .with_output("out_color", Some(DepthMode::LessEqualWrite))
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
            ambient,
            mesh_storage,
            tex_storage,
            material_defaults,
            mesh,
            material,
            global,
            light,
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

        set_light_args(effect, encoder, &light, &ambient, camera);

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

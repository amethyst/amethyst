//! Forward physically-based drawing pass.

use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use amethyst_core::transform::GlobalTransform;
use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use specs::{Fetch, Join, ReadStorage};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use light::Light;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pass::shaded_util::{set_light_args, setup_light_buffers};
use pass::util::{draw_mesh, get_camera, setup_textures, setup_vertex_args};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassData};
use resources::AmbientColor;
use tex::Texture;
use transparent::{Transparent, TransparentBackToFront};
use types::{Encoder, Factory};
use vertex::{Normal, Position, Query, Tangent, TexCoord};

/// Draw mesh with physically based lighting
/// `V` is `VertexFormat`
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "V: Query<(Position, Normal, Tangent, TexCoord)>"))]
pub struct DrawPbm<V> {
    _pd: PhantomData<V>,
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl<V> DrawPbm<V>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
{
    /// Create instance of `DrawPbm` pass
    pub fn new() -> Self {
        Default::default()
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

impl<'a, V> PassData<'a> for DrawPbm<V>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
{
    type Data = (
        Option<Fetch<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Fetch<'a, AmbientColor>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        Fetch<'a, TransparentBackToFront>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Light>,
        ReadStorage<'a, Transparent>,
    );
}

impl<V> Pass for DrawPbm<V>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
{
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);
        setup_vertex_args(&mut builder);
        setup_light_buffers(&mut builder);
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
            active,
            camera,
            ambient,
            mesh_storage,
            tex_storage,
            material_defaults,
            back_to_front,
            mesh,
            material,
            global,
            light,
            transparent,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        set_light_args(effect, encoder, &light, &ambient, camera);

        for (mesh, material, global, _) in (&mesh, &material, &global, !&transparent).join() {
            draw_mesh(
                encoder,
                effect,
                false,
                mesh_storage.get(mesh),
                None,
                &*tex_storage,
                Some(material),
                &*material_defaults,
                camera,
                Some(global),
                &[V::QUERIED_ATTRIBUTES],
                &TEXTURES,
            );
        }

        for entity in &back_to_front.entities {
            if let Some(mesh) = mesh.get(*entity) {
                draw_mesh(
                    encoder,
                    effect,
                    false,
                    mesh_storage.get(mesh),
                    None,
                    &*tex_storage,
                    material.get(*entity),
                    &*material_defaults,
                    camera,
                    global.get(*entity),
                    &[V::QUERIED_ATTRIBUTES],
                    &TEXTURES,
                );
            }
        }
    }
}

//! Forward physically-based drawing pass.

use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    specs::prelude::{Join, Read, ReadExpect, ReadStorage},
    transform::GlobalTransform,
};

use crate::{
    cam::{ActiveCamera, Camera},
    error::Result,
    hidden::{Hidden, HiddenPropagate},
    light::Light,
    mesh::{Mesh, MeshHandle},
    mtl::{Material, MaterialDefaults},
    pass::{
        shaded_util::{set_light_args, setup_light_buffers},
        skinning::{create_skinning_effect, setup_skinning_buffers},
        util::{draw_mesh, get_camera, setup_textures, setup_vertex_args},
    },
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    resources::AmbientColor,
    skinning::JointTransforms,
    tex::Texture,
    types::{Encoder, Factory},
    vertex::{Attributes, Normal, Position, Separate, Tangent, TexCoord, VertexFormat},
    visibility::Visibility,
    Rgba,
};

use super::*;

static ATTRIBUTES: [Attributes<'static>; 4] = [
    Separate::<Position>::ATTRIBUTES,
    Separate::<Normal>::ATTRIBUTES,
    Separate::<Tangent>::ATTRIBUTES,
    Separate::<TexCoord>::ATTRIBUTES,
];

/// Draw mesh with physically based lighting
///
/// See the [crate level documentation](index.html) for information about interleaved and separate
/// passes.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct DrawPbmSeparate {
    skinning: bool,
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
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

impl<'a> PassData<'a> for DrawPbmSeparate {
    type Data = (
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        Read<'a, AmbientColor>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
        ReadExpect<'a, MaterialDefaults>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Light>,
        ReadStorage<'a, JointTransforms>,
        ReadStorage<'a, Rgba>,
    );
}

impl Pass for DrawPbmSeparate {
    fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect> {
        #[cfg(feature = "profiler")]
        profile_scope!("render_pass_pbm_compile");

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
            visibility,
            hidden,
            hidden_prop,
            mesh,
            material,
            global,
            light,
            joints,
            rgba,
        ): <Self as PassData<'a>>::Data,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("render_pass_pbm_apply");

        let camera = get_camera(active, &camera, &global);

        set_light_args(effect, encoder, &light, &global, &ambient, camera);

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
                        &TEXTURES,
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
                        rgba,
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
                            rgba.get(*entity),
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

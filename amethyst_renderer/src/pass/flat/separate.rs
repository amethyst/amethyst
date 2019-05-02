//! Simple flat forward drawing pass.

use derivative::Derivative;
use gfx::pso::buffer::ElemStride;
use gfx::traits::Pod;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;
use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    alga::general::SubsetOf,
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage},
    math::RealField,
    transform::Transform,
};
use amethyst_error::Error;

use crate::{
    cam::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    mesh::{Mesh, MeshHandle},
    mtl::{Material, MaterialDefaults},
    pass::{
        skinning::{create_skinning_effect, setup_skinning_buffers},
        util::{default_transparency, draw_mesh, get_camera, setup_textures, VertexArgs},
    },
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    skinning::JointTransforms,
    tex::Texture,
    types::{Encoder, Factory},
    vertex::{Attributes, Position, Separate, TexCoord, VertexFormat},
    visibility::Visibility,
    Rgba,
};

use super::*;

static ATTRIBUTES: [Attributes<'static>; 2] = [
    Separate::<Position>::ATTRIBUTES,
    Separate::<TexCoord>::ATTRIBUTES,
];

/// Draw mesh without lighting
///
/// See the [crate level documentation](index.html) for information about interleaved and separate
/// passes.
///
/// # Type Parameters
///
/// * `N`: `RealBound` (f32, f64)
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawFlatSeparate<N> {
    skinning: bool,
    #[derivative(Default(value = "default_transparency()"))]
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
    _pd: PhantomData<N>,
}

impl<N> DrawFlatSeparate<N>
where
    Self: Pass,
    N: RealField,
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

    /// Transparency is enabled by default.
    /// If you pass false to this function transparency will be disabled.
    ///
    /// If you pass true and this was disabled previously default settings will be reinstated.
    /// If you pass true and this was already enabled this will do nothing.
    pub fn with_transparency(mut self, input: bool) -> Self {
        if input {
            if self.transparency.is_none() {
                self.transparency = default_transparency();
            }
        } else {
            self.transparency = None;
        }
        self
    }

    /// Set transparency settings to custom values.
    pub fn with_transparency_settings(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }
}

impl<'a, N: RealField> PassData<'a> for DrawFlatSeparate<N> {
    type Data = (
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
        ReadExpect<'a, MaterialDefaults>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, Transform<N>>,
        ReadStorage<'a, JointTransforms<N>>,
        ReadStorage<'a, Rgba>,
    );
}

impl<N: RealField + SubsetOf<f32> + Pod> Pass for DrawFlatSeparate<N> {
    fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect, Error> {
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
            transform,
            joints,
            rgba,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &transform);

        match visibility {
            None => {
                for (joint, mesh, material, transform, rgba, _, _) in (
                    joints.maybe(),
                    &mesh,
                    &material,
                    &transform,
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
                        Some(transform),
                        &ATTRIBUTES,
                        &TEXTURES,
                    );
                }
            }
            Some(ref visibility) => {
                for (joint, mesh, material, transform, rgba, _) in (
                    joints.maybe(),
                    &mesh,
                    &material,
                    &transform,
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
                        Some(transform),
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
                            transform.get(*entity),
                            &ATTRIBUTES,
                            &TEXTURES,
                        );
                    }
                }
            }
        }
    }
}

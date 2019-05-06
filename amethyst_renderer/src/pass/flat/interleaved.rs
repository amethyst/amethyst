//! Simple flat forward drawing pass.

use gfx::traits::Pod;
use std::marker::PhantomData;

use derivative::Derivative;
use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;

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
    pass::util::{default_transparency, draw_mesh, get_camera, setup_textures, VertexArgs},
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    tex::Texture,
    types::{Encoder, Factory},
    vertex::{Position, Query, TexCoord},
    visibility::Visibility,
    Rgba,
};

use super::*;

/// Draw mesh without lighting
///
/// See the [crate level documentation](index.html) for information about interleaved and separate
/// passes.
///
/// # Type Parameters
///
/// * `V`: `VertexFormat`
/// * `N`: `RealBound` (f32, f64)
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "V: Query<(Position, TexCoord)>, Self: Pass"))]
pub struct DrawFlat<V, N> {
    _marker: PhantomData<(V, N)>,
    #[derivative(Default(value = "default_transparency()"))]
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl<V, N> DrawFlat<V, N>
where
    V: Query<(Position, TexCoord)>,
    N: RealField,
    Self: Pass,
{
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        Default::default()
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

impl<'a, V, N> PassData<'a> for DrawFlat<V, N>
where
    V: Query<(Position, TexCoord)>,
    N: RealField,
{
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
        ReadStorage<'a, Rgba>,
    );
}

impl<V, N> Pass for DrawFlat<V, N>
where
    V: Query<(Position, TexCoord)>,
    N: RealField + SubsetOf<f32> + Pod,
{
    fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect, Error> {
        use std::mem;
        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder
            .with_raw_constant_buffer(
                "VertexArgs",
                mem::size_of::<<VertexArgs as Uniform>::Std140>(),
                1,
            )
            .with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);
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
            rgba,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &transform);

        match visibility {
            None => {
                for (mesh, material, transform, rgba, _, _) in (
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
                        false,
                        mesh_storage.get(mesh),
                        None,
                        &tex_storage,
                        Some(material),
                        &material_defaults,
                        rgba,
                        camera,
                        Some(transform),
                        &[V::QUERIED_ATTRIBUTES],
                        &TEXTURES,
                    );
                }
            }
            Some(ref visibility) => {
                for (mesh, material, transform, rgba, _) in (
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
                        false,
                        mesh_storage.get(mesh),
                        None,
                        &tex_storage,
                        Some(material),
                        &material_defaults,
                        rgba,
                        camera,
                        Some(transform),
                        &[V::QUERIED_ATTRIBUTES],
                        &TEXTURES,
                    );
                }

                for entity in &visibility.visible_ordered {
                    if let Some(mesh) = mesh.get(*entity) {
                        draw_mesh(
                            encoder,
                            effect,
                            false,
                            mesh_storage.get(mesh),
                            None,
                            &tex_storage,
                            material.get(*entity),
                            &material_defaults,
                            rgba.get(*entity),
                            camera,
                            transform.get(*entity),
                            &[V::QUERIED_ATTRIBUTES],
                            &TEXTURES,
                        );
                    }
                }
            }
        }
    }
}

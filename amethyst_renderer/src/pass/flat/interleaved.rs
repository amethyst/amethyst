//! Simple flat forward drawing pass.

use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use cgmath::{Matrix4, One};
use gfx::pso::buffer::ElemStride;

use rayon::iter::ParallelIterator;
use rayon::iter::internal::UnindexedConsumer;
use specs::{Component, Fetch, ParJoin, ReadStorage};

use cam::Camera;
use error::Result;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pipe::pass::{Pass, PassApply, PassData, Supplier};
use pipe::{DepthMode, Effect, NewEffect};
use tex::Texture;
use types::Encoder;
use vertex::{Position, Query, TexCoord};
use super::*;

/// Draw mesh without lighting
/// `V` is `VertexFormat`
/// `T` is transform matrix component
#[derive(Clone, Debug, PartialEq)]
pub struct DrawFlat<V, T> {
    _pd: PhantomData<(V, T)>,
}

impl<V, T> DrawFlat<V, T>
where
    V: Query<(Position, TexCoord)>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    Self: Pass,
{
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        DrawFlat { _pd: PhantomData }
    }
}

impl<'a, V, T> PassData<'a> for DrawFlat<V, T>
where
    V: Query<(Position, TexCoord)>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    type Data = (
        Option<Fetch<'a, Camera>>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, T>,
    );
}

impl<'a, V, T> PassApply<'a> for DrawFlat<V, T>
where
    V: Query<(Position, TexCoord)>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    type Apply = DrawFlatApply<'a, V, T>;
}

impl<V, T> Pass for DrawFlat<V, T>
where
    V: Query<(Position, TexCoord)>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0)
            .with_texture("albedo")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        supplier: Supplier<'a>,
        (camera, mesh_storage, tex_storage, material_defaults, mesh, material, global): (
            Option<Fetch<'b, Camera>>,
            Fetch<'a, AssetStorage<Mesh>>,
            Fetch<'a, AssetStorage<Texture>>,
            Fetch<'a, MaterialDefaults>,
            ReadStorage<'b, MeshHandle>,
            ReadStorage<'b, Material>,
            ReadStorage<'b, T>,
        ),
    ) -> DrawFlatApply<'a, V, T> {
        DrawFlatApply {
            camera,
            mesh_storage,
            tex_storage,
            material_defaults,
            mesh,
            material,
            global,
            supplier,
            pd: PhantomData,
        }
    }
}

pub struct DrawFlatApply<'a, V, T: Component> {
    camera: Option<Fetch<'a, Camera>>,
    mesh_storage: Fetch<'a, AssetStorage<Mesh>>,
    tex_storage: Fetch<'a, AssetStorage<Texture>>,
    material_defaults: Fetch<'a, MaterialDefaults>,
    mesh: ReadStorage<'a, MeshHandle>,
    material: ReadStorage<'a, Material>,
    global: ReadStorage<'a, T>,
    supplier: Supplier<'a>,
    pd: PhantomData<V>,
}

impl<'a, V, T> ParallelIterator for DrawFlatApply<'a, V, T>
where
    V: Query<(Position, TexCoord)>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let DrawFlatApply {
            camera,
            mesh_storage,
            tex_storage,
            material_defaults,
            mesh,
            material,
            global,
            supplier,
            ..
        } = self;

        let camera = &camera;
        let mesh_storage = &mesh_storage;
        let tex_storage = &tex_storage;
        let material_defaults = &material_defaults;

        supplier
            .supply((&mesh, &material, &global).par_join().map(
                move |(mesh, material, global)| {
                    move |encoder: &mut Encoder, effect: &mut Effect| if let Some(mesh) =
                        mesh_storage.get(mesh)
                    {
                        let vbuf = match mesh.buffer(V::QUERIED_ATTRIBUTES) {
                            Some(vbuf) => vbuf.clone(),
                            None => return,
                        };

                        let vertex_args = camera
                            .as_ref()
                            .map(|cam| {
                                VertexArgs {
                                    proj: cam.proj.into(),
                                    view: cam.to_view_matrix().into(),
                                    model: *global.as_ref(),
                                }
                            })
                            .unwrap_or_else(|| {
                                VertexArgs {
                                    proj: Matrix4::one().into(),
                                    view: Matrix4::one().into(),
                                    model: *global.as_ref(),
                                }
                            });

                        let albedo = tex_storage
                            .get(&material.albedo)
                            .or_else(|| tex_storage.get(&material_defaults.0.albedo))
                            .unwrap();

                        effect.update_constant_buffer("VertexArgs", &vertex_args, encoder);
                        effect.data.textures.push(albedo.view().clone());
                        effect.data.samplers.push(albedo.sampler().clone());

                        effect.data.vertex_bufs.push(vbuf);

                        effect.draw(mesh.slice(), encoder);
                    }
                },
            ))
            .drive_unindexed(consumer)
    }
}

//! Simple flat forward drawing pass.

use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use amethyst_core::transform::Transform;
use cgmath::{Matrix4, One};
use gfx::pso::buffer::ElemStride;

use rayon::iter::ParallelIterator;
use rayon::iter::internal::UnindexedConsumer;
use specs::{Fetch, ParJoin, ReadStorage};

use super::*;
use cam::Camera;
use error::Result;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassApply, PassData, Supplier};
use tex::Texture;
use types::Encoder;
use vertex::{Position, Query, TexCoord};

/// Draw mesh without lighting
/// `V` is `VertexFormat`
#[derive(Clone, Debug, PartialEq)]
pub struct DrawFlat<V> {
    _pd: PhantomData<V>,
}

impl<V> DrawFlat<V>
where
    V: Query<(Position, TexCoord)>,
    Self: Pass,
{
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        DrawFlat { _pd: PhantomData }
    }
}

impl<'a, V> PassData<'a> for DrawFlat<V>
where
    V: Query<(Position, TexCoord)>,
{
    type Data = (
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        Fetch<'a, Orientation>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, ActiveCamera>,
    );
}

impl<'a, V> PassApply<'a> for DrawFlat<V>
where
    V: Query<(Position, TexCoord)>,
{
    type Apply = DrawFlatApply<'a, V>;
}

impl<V> Pass for DrawFlat<V>
where
    V: Query<(Position, TexCoord)>,
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
        (mesh_storage, tex_storage, material_defaults, orientation, mesh, material, global, camera, active_camera): (
            Fetch<'a, AssetStorage<Mesh>>,
            Fetch<'a, AssetStorage<Texture>>,
            Fetch<'a, MaterialDefaults>,
            Fetch<'a, Orientation>,
            ReadStorage<'b, MeshHandle>,
            ReadStorage<'b, Material>,
            ReadStorage<'b, Transform>,
            ReadStorage<'a, Camera>,
            ReadStorage<'a, ActiveCamera>,
        ),
    ) -> DrawFlatApply<'a, V> {
        DrawFlatApply {
            mesh_storage,
            tex_storage,
            material_defaults,
            orientation
            mesh,
            material,
            global,
            camera,
            active_camera
            supplier,
            pd: PhantomData,
        }
    }
}

pub struct DrawFlatApply<'a, V> {
    mesh_storage: Fetch<'a, AssetStorage<Mesh>>,
    tex_storage: Fetch<'a, AssetStorage<Texture>>,
    material_defaults: Fetch<'a, MaterialDefaults>,
    orientation: Fetch<'a, Orientation>,
    mesh: ReadStorage<'a, MeshHandle>,
    material: ReadStorage<'a, Material>,
    global: ReadStorage<'a, Transform>,
    ReadStorage<'a, Camera>,
    ReadStorage<'a, ActiveCamera>,
    supplier: Supplier<'a>,
    pd: PhantomData<V>,
}

impl<'a, V> ParallelIterator for DrawFlatApply<'a, V>
where
    V: Query<(Position, TexCoord)>,
{
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let DrawFlatApply {
            mesh_storage,
            tex_storage,
            material_defaults,
            orientation,
            mesh,
            material,
            global,
            camera,
            active_camera,
            supplier,
            ..
        } = self;

        // (_, current_camera, camera_trans)
        let cam = (active_camera, camera, global).join().next();
        let (proj_matrix, view_matrix) = {
            if active_camera.is_some() {
                let (_, camera, trans) = active_camera.unwrap();

                (camera.proj, trans.to_view_matrix(orientation))
            }
            else {
                (Matrix4::one(), Matrix4::one())
            }
        };

        let proj_matrix = &proj_matrix;
        let view_matrix = &view_matrix;
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

                        let vertex_args = VertexArgs {
                            proj: proj_matrix.into(),
                            view: view_matrix.into(),
                            model: *global.as_ref(),
                        };

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

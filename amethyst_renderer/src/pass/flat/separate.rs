//! Simple flat forward drawing pass.

use amethyst_assets::AssetStorage;
use amethyst_core::transform::Transform;
use cgmath::{Matrix4, One, SquareMatrix};
use gfx::pso::buffer::ElemStride;

use rayon::iter::ParallelIterator;
use rayon::iter::internal::UnindexedConsumer;
use specs::{Fetch, Join, ParJoin, ReadStorage};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassApply, PassData, Supplier};
use tex::Texture;
use types::Encoder;
use vertex::{Position, Separate, TexCoord, VertexFormat};

/// Draw mesh without lighting
#[derive(Clone, Debug, PartialEq)]
pub struct DrawFlatSeparate;

impl DrawFlatSeparate
where
    Self: Pass,
{
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        DrawFlatSeparate {}
    }
}

impl<'a> PassData<'a> for DrawFlatSeparate {
    type Data = (
        Option<Fetch<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, Transform>,
    );
}

impl<'a> PassApply<'a> for DrawFlatSeparate {
    type Apply = DrawFlatSeparateApply<'a>;
}

impl Pass for DrawFlatSeparate {
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_vertex_buffer(
                Separate::<Position>::ATTRIBUTES,
                Separate::<Position>::size() as ElemStride,
                0,
            )
            .with_raw_vertex_buffer(
                Separate::<TexCoord>::ATTRIBUTES,
                Separate::<TexCoord>::size() as ElemStride,
                0,
            )
            .with_texture("albedo")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        supplier: Supplier<'a>,
        (active, camera, mesh_storage, tex_storage, material_defaults, mesh, material, global): (
            Option<Fetch<'a, ActiveCamera>>,
            ReadStorage<'a, Camera>,
            Fetch<'a, AssetStorage<Mesh>>,
            Fetch<'a, AssetStorage<Texture>>,
            Fetch<'a, MaterialDefaults>,
            ReadStorage<'b, MeshHandle>,
            ReadStorage<'b, Material>,
            ReadStorage<'b, Transform>,
        ),
    ) -> DrawFlatSeparateApply<'a> {
        DrawFlatSeparateApply {
            active,
            camera,
            mesh_storage,
            tex_storage,
            material_defaults,
            mesh,
            material,
            global,
            supplier,
        }
    }
}

pub struct DrawFlatSeparateApply<'a> {
    active: Option<Fetch<'a, ActiveCamera>>,
    camera: ReadStorage<'a, Camera>,
    mesh_storage: Fetch<'a, AssetStorage<Mesh>>,
    tex_storage: Fetch<'a, AssetStorage<Texture>>,
    material_defaults: Fetch<'a, MaterialDefaults>,
    mesh: ReadStorage<'a, MeshHandle>,
    material: ReadStorage<'a, Material>,
    global: ReadStorage<'a, Transform>,
    supplier: Supplier<'a>,
}

impl<'a> ParallelIterator for DrawFlatSeparateApply<'a> {
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let DrawFlatSeparateApply {
            active,
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

        let camera: Option<(&Camera, &Transform)> = active
            .and_then(|a| {
                let cam = camera.get(a.entity);
                let transform = global.get(a.entity);
                cam.into_iter().zip(transform.into_iter()).next()
            })
            .or_else(|| (&camera, &global).join().next());

        let mesh_storage = &mesh_storage;
        let tex_storage = &tex_storage;
        let material_defaults = &material_defaults;

        supplier
            .supply((&mesh, &material, &global).par_join().map(
                move |(mesh, material, global)| {
                    move |encoder: &mut Encoder, effect: &mut Effect| if let Some(mesh) =
                        mesh_storage.get(mesh)
                    {
                        for attrs in [
                            Separate::<Position>::ATTRIBUTES,
                            Separate::<TexCoord>::ATTRIBUTES,
                        ].iter()
                        {
                            match mesh.buffer(attrs) {
                                Some(vbuf) => effect.data.vertex_bufs.push(vbuf.clone()),
                                None => return,
                            }
                        }

                        let vertex_args = camera
                            .as_ref()
                            .map(|&(ref cam, ref transform)| {
                                VertexArgs {
                                    proj: cam.proj.into(),
                                    view: Matrix4::from(transform.0).invert().unwrap().into(),
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

                        effect.draw(mesh.slice(), encoder);
                    }
                },
            ))
            .drive_unindexed(consumer)
    }
}

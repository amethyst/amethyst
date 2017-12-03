//! Simple flat forward drawing pass.

use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use amethyst_core::cgmath::{Matrix4, One, SquareMatrix};
use amethyst_core::transform::Transform;
use gfx::pso::buffer::ElemStride;
use specs::{Fetch, Join, ReadStorage};

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassData};
use tex::Texture;
use types::{Encoder, Factory};
use vertex::{Position, Query, TexCoord};

/// Draw mesh without lighting
/// `V` is `VertexFormat`
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "V: Query<(Position, TexCoord)>, Self: Pass"))]
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
        Default::default()
    }
}

impl<'a, V> PassData<'a> for DrawFlat<V>
where
    V: Query<(Position, TexCoord)>,
{
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
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
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
    ) {
        let camera: Option<(&Camera, &Transform)> = active
            .and_then(|a| {
                let cam = camera.get(a.entity);
                let transform = global.get(a.entity);
                cam.into_iter().zip(transform.into_iter()).next()
            })
            .or_else(|| (&camera, &global).join().next());

        for (mesh, material, global) in (&mesh, &material, &global).join() {
            let mesh = match mesh_storage.get(mesh) {
                Some(mesh) => mesh,
                None => continue,
            };
            let vbuf = match mesh.buffer(V::QUERIED_ATTRIBUTES) {
                Some(vbuf) => vbuf.clone(),
                None => continue,
            };

            let vertex_args = camera
                .as_ref()
                .map(|&(ref cam, ref transform)| {
                    VertexArgs {
                        proj: cam.proj.into(),
                        view: transform.0.invert().unwrap().into(),
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
            effect.clear();
        }
    }
}

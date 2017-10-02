//! Simple flat forward drawing pass.

use std::marker::PhantomData;

use cgmath::{Matrix4, One};
use gfx::pso::buffer::ElemStride;

use rayon::iter::ParallelIterator;
use rayon::iter::internal::UnindexedConsumer;
use specs::{Component, Fetch, ParJoin, ReadStorage};

use cam::Camera;
use error::Result;
use mesh::Mesh;
use mtl::Material;
use pipe::pass::{Pass, PassApply, PassData, Supplier};
use pipe::{DepthMode, Effect, NewEffect};
use types::Encoder;
use vertex::{Attribute, Position, TextureCoord, VertexFormat, WithField};

static VERT_SRC: &[u8] = include_bytes!("shaders/vertex/basic.glsl");
static FRAG_SRC: &[u8] = include_bytes!("shaders/fragment/flat.glsl");

/// Draw mesh without lighting
/// `V` is `VertexFormat`
/// `M` is `Mesh` component
/// `N` is `Material` component
/// `T` is transform matrix component
#[derive(Clone, Debug, PartialEq)]
pub struct DrawFlat<V, M, N, T> {
    vertex_attributes: [(&'static str, Attribute); 2],
    _pd: PhantomData<(V, M, N, T)>,
}

impl<V, M, N, T> DrawFlat<V, M, N, T>
where
    V: VertexFormat + WithField<Position> + WithField<TextureCoord>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
    Self: Pass,
{
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        DrawFlat {
            vertex_attributes: [
                ("position", V::attribute::<Position>()),
                ("tex_coord", V::attribute::<TextureCoord>()),
            ],
            _pd: PhantomData,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct VertexArgs {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

impl<'a, V, M, N, T> PassData<'a> for DrawFlat<V, M, N, T>
where
    V: VertexFormat + WithField<Position> + WithField<TextureCoord>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
{
    type Data = (
        Option<Fetch<'a, Camera>>,
        ReadStorage<'a, M>,
        ReadStorage<'a, N>,
        ReadStorage<'a, T>,
    );
}

impl<'a, V, M, N, T> PassApply<'a> for DrawFlat<V, M, N, T>
where
    V: VertexFormat + WithField<Position> + WithField<TextureCoord>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
{
    type Apply = DrawFlatApply<'a, V, M, N, T>;
}

impl<V, M, N, T> Pass for DrawFlat<V, M, N, T>
where
    V: VertexFormat + WithField<Position> + WithField<TextureCoord>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
{
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_vertex_buffer(self.vertex_attributes.as_ref(), V::size() as ElemStride, 0)
            .with_texture("albedo")
            .with_output("color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        supplier: Supplier<'a>,
        (camera, mesh, material, global): (
            Option<Fetch<'b, Camera>>,
            ReadStorage<'b, M>,
            ReadStorage<'b, N>,
            ReadStorage<'b, T>,
        ),
    ) -> DrawFlatApply<'a, V, M, N, T> {
        DrawFlatApply {
            camera: camera,
            mesh: mesh,
            material: material,
            global: global,
            supplier: supplier,
            pd: PhantomData,
        }
    }
}

pub struct DrawFlatApply<'a, V, M: Component, N: Component, T: Component> {
    camera: Option<Fetch<'a, Camera>>,
    mesh: ReadStorage<'a, M>,
    material: ReadStorage<'a, N>,
    global: ReadStorage<'a, T>,
    supplier: Supplier<'a>,
    pd: PhantomData<V>,
}

impl<'a, V, M, N, T> ParallelIterator for DrawFlatApply<'a, V, M, N, T>
where
    V: VertexFormat + WithField<Position> + WithField<TextureCoord>,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
{
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let DrawFlatApply {
            camera,
            mesh,
            material,
            global,
            supplier,
            ..
        } = self;

        let camera = &camera;

        supplier
            .supply((&mesh, &material, &global).par_join().map(
                move |(mesh, material, global)| {
                    move |encoder: &mut Encoder, effect: &mut Effect| {
                        let mesh = mesh.as_ref();
                        let material = material.as_ref();

                        if mesh.attributes() != V::attributes().as_ref() {
                            return;
                        }

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

                        effect.update_constant_buffer("VertexArgs", &vertex_args, encoder);
                        effect.data.textures.push(material.albedo.view().clone());

                        effect.data.samplers.push(material.albedo.sampler().clone());

                        effect.draw(mesh, encoder);
                    }
                },
            ))
            .drive_unindexed(consumer)
    }
}

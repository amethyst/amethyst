//! Simple flat forward drawing pass.

use std::marker::PhantomData;

use amethyst_assets::{AssetStorage, Loader};
use cgmath::{Matrix4, Ortho};
use gfx::pso::buffer::ElemStride;
use hibitset::BitSet;

use rayon::iter::ParallelIterator;
use rayon::iter::internal::UnindexedConsumer;
use specs::{Fetch, FetchMut, Join, ParJoin, ReadStorage, WriteStorage};

use super::*;
use amethyst_renderer::Result;
use amethyst_renderer::resources::ScreenDimensions;
use amethyst_renderer::mesh::{Mesh, MeshHandle};
use amethyst_renderer::pipe::{DepthMode, Effect, NewEffect};
use amethyst_renderer::pipe::pass::{Pass, PassApply, PassData, Supplier};
use amethyst_renderer::Texture;
use amethyst_renderer::Encoder;
use amethyst_renderer::vertex::{PosTex, Position, Query, TexCoord};

const VERT_SRC: &[u8] = include_bytes!("vertex.glsl");
const FRAG_SRC: &[u8] = include_bytes!("frag.glsl");

#[derive(Copy, Clone)]
#[allow(dead_code)] // This is used by the shaders
struct VertexArgs {
    proj: [[f32; 4]; 4],
    coord: [f32; 2],
    dimension: [f32; 2],
}

/// Draw mesh without lighting
/// `V` is `VertexFormat`
#[derive(Clone, Debug, PartialEq)]
pub struct DrawUi<V> {
    mesh_handle: MeshHandle,
    _pd: PhantomData<V>,
}

impl<V> DrawUi<V>
where
    V: Query<(Position, TexCoord)>,
    Self: Pass,
{
    /// Create instance of `DrawUi` pass
    pub fn new(loader: &Loader, mesh_storage: &AssetStorage<Mesh>) -> Self {
        /// Initialize a single unit quad, we'll use this mesh when drawing quads later
        let data = vec![
            PosTex {
                position: [0., 1., 0.],
                tex_coord: [0. , 0.],
            },
            PosTex {
                position: [1., 1., 0.],
                tex_coord: [1. , 0.],
            },
            PosTex {
                position: [0., 0., 0.],
                tex_coord: [1., 1.],
            },
            PosTex {
                position: [1., 0., 0.],
                tex_coord: [1., 1.],
            },
            PosTex {
                position: [0., 0., 0.],
                tex_coord: [0., 1.],
            },
            PosTex {
                position: [1., 0., 0.],
                tex_coord: [0., 0.],
            }
        ].into();
        let mesh_handle = loader.load_from_data(data, mesh_storage);
        DrawUi {
            mesh_handle,
            _pd: PhantomData
        }
    }
}

impl<'a, V> PassData<'a> for DrawUi<V>
where
    V: Query<(Position, TexCoord)>,
{
    type Data = (
        Fetch<'a, ScreenDimensions>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        ReadStorage<'a, UiImage>,
        WriteStorage<'a, UiTransform>,
    );
}

impl<'a, V> PassApply<'a> for DrawUi<V>
where
    V: Query<(Position, TexCoord)>,
{
    type Apply = DrawUiApply<'a, V>;
}

impl<V> Pass for DrawUi<V>
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
        (screen_dimensions, mesh_storage, tex_storage, ui_image, ui_transform): (
            Fetch<'a, ScreenDimensions>,
            Fetch<'a, AssetStorage<Mesh>>,
            Fetch<'a, AssetStorage<Texture>>,
            ReadStorage<'a, UiImage>,
            WriteStorage<'a, UiTransform>,
        ),
    ) -> DrawUiApply<'a, V> {
        DrawUiApply {
            screen_dimensions,
            mesh_storage,
            tex_storage,
            ui_image,
            ui_transform,
            unit_mesh: self.mesh_handle.clone(),
            supplier,
            pd: PhantomData,
        }
    }
}

pub struct DrawUiApply<'a, V> {
    screen_dimensions: Fetch<'a, ScreenDimensions>,
    mesh_storage: Fetch<'a, AssetStorage<Mesh>>,
    tex_storage: Fetch<'a, AssetStorage<Texture>>,
    ui_image: ReadStorage<'a, UiImage>,
    ui_transform: WriteStorage<'a, UiTransform>,
    unit_mesh: MeshHandle,
    supplier: Supplier<'a>,
    pd: PhantomData<V>,
}

impl<'a, V> ParallelIterator for DrawUiApply<'a, V>
where
    V: Query<(Position, TexCoord)>,
{
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let DrawUiApply {
            screen_dimensions,
            mesh_storage,
            tex_storage,
            ui_image,
            ui_transform,
            unit_mesh,
            supplier,
            ..
        } = self;

        let screen_dimensions = &screen_dimensions;
        let mesh_storage = &mesh_storage;
        let tex_storage = &tex_storage;
        let ui_image = &ui_image;
        let ui_transform = &ui_transform;
        let unit_mesh = &unit_mesh;

        /// This pass can't be executed in parallel, so we use a dumby bitset of a
        /// single element to provide a fake parallel iterator that performs the entire
        /// pass in the first iteration.
        let mut bitset = BitSet::new();
        bitset.add(0);

        let proj = Ortho {
            left: -screen_dimensions.width() / 2.0,
            right: screen_dimensions.width() / 2.0,
            bottom: -screen_dimensions.height() / 2.0,
            top: screen_dimensions.height() / 2.0,
            near: 0.1,
            far: 2000.0,
        };



        supplier
            .supply(bitset.par_join().map(
                move |_id| {
                    move |encoder: &mut Encoder, effect: &mut Effect|
                    for (ui_transform, ui_image) in (ui_transform, ui_image).join() {
                        if let Some(mesh) = mesh_storage.get(unit_mesh)
                        {
                            let vbuf = match mesh.buffer(V::QUERIED_ATTRIBUTES) {
                                Some(vbuf) => vbuf.clone(),
                                None => continue,
                            };

                            if let Some(image) = tex_storage.get(&ui_image.texture) {
                                let proj: Matrix4<f32> = proj.into();

                                let vertex_args = VertexArgs {
                                    proj: proj.into(),
                                    coord: [ui_transform.x, ui_transform.y],
                                    dimension: [ui_transform.width, ui_transform.height],
                                };

                                effect.update_constant_buffer("VertexArgs", &vertex_args, encoder);
                                effect.data.textures.push(image.view().clone());
                                effect.data.samplers.push(image.sampler().clone());

                                effect.data.vertex_bufs.push(vbuf);

                                effect.draw(mesh.slice(), encoder);
                            } else {
                                eprintln!("Unable to draw UI image, image handle is invalid.");
                            }
                        }
                    }
                },
            ))
            .drive_unindexed(consumer)
    }
}

//! Simple flat forward drawing pass.

use std::cmp::{Ordering, PartialOrd};

use amethyst_assets::{AssetStorage, Loader};
use cgmath::{Matrix4, Ortho};
use gfx::preset::blend;
use gfx::pso::buffer::ElemStride;
use gfx::state::ColorMask;
use hibitset::BitSet;

use rayon::iter::ParallelIterator;
use rayon::iter::internal::UnindexedConsumer;
use specs::{Entities, Entity, Fetch, Join, ParJoin, ReadStorage};

use super::*;
use amethyst_renderer::Encoder;
use amethyst_renderer::Result;
use amethyst_renderer::Texture;
use amethyst_renderer::VertexFormat;
use amethyst_renderer::mesh::{Mesh, MeshHandle};
use amethyst_renderer::pipe::{Effect, NewEffect};
use amethyst_renderer::pipe::pass::{Pass, PassApply, PassData, Supplier};
use amethyst_renderer::resources::ScreenDimensions;
use amethyst_renderer::vertex::PosTex;

const VERT_SRC: &[u8] = include_bytes!("vertex.glsl");
const FRAG_SRC: &[u8] = include_bytes!("frag.glsl");

#[derive(Copy, Clone)]
#[allow(dead_code)] // This is used by the shaders
#[repr(C)]
struct VertexArgs {
    proj: [[f32; 4]; 4],
    coord: [f32; 2],
    dimension: [f32; 2],
}

/// Draw Ui elements, this uses target with name "amethyst_ui"
/// `V` is `VertexFormat`
#[derive(Clone, Debug, PartialEq)]
pub struct DrawUi {
    mesh_handle: MeshHandle,
    cached_draw_order: Vec<(f32, Entity)>,
}

impl DrawUi
where
    Self: Pass,
{
    /// Create instance of `DrawUi` pass
    pub fn new(loader: &Loader, mesh_storage: &AssetStorage<Mesh>) -> Self {
        // Initialize a single unit quad, we'll use this mesh when drawing quads later
        let data = vec![
            PosTex {
                position: [0., 1., 0.],
                tex_coord: [0., 0.],
            },
            PosTex {
                position: [1., 1., 0.],
                tex_coord: [1., 0.],
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
            },
        ].into();
        let mesh_handle = loader.load_from_data(data, mesh_storage);
        DrawUi {
            mesh_handle,
            cached_draw_order: Vec::new(),
        }
    }
}

impl<'a> PassData<'a> for DrawUi {
    type Data = (
        Entities<'a>,
        Fetch<'a, ScreenDimensions>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        ReadStorage<'a, UiImage>,
        ReadStorage<'a, UiTransform>,
    );
}

impl<'a> PassApply<'a> for DrawUi {
    type Apply = DrawUiApply<'a>;
}

impl Pass for DrawUi {
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_vertex_buffer(PosTex::ATTRIBUTES, PosTex::size() as ElemStride, 0)
            .with_texture("albedo")
            .with_blended_output("color", ColorMask::all(), blend::ALPHA, None)
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        supplier: Supplier<'a>,
        (entities, screen_dimensions, mesh_storage, tex_storage, ui_image, ui_transform): (
            Entities<'a>,
            Fetch<'a, ScreenDimensions>,
            Fetch<'a, AssetStorage<Mesh>>,
            Fetch<'a, AssetStorage<Texture>>,
            ReadStorage<'a, UiImage>,
            ReadStorage<'a, UiTransform>,
        ),
    ) -> DrawUiApply<'a> {
        DrawUiApply {
            entities,
            screen_dimensions,
            mesh_storage,
            tex_storage,
            ui_image,
            ui_transform,
            unit_mesh: self.mesh_handle.clone(),
            cached_draw_order: &mut self.cached_draw_order,
            supplier,
        }
    }
}

pub struct DrawUiApply<'a> {
    entities: Entities<'a>,
    screen_dimensions: Fetch<'a, ScreenDimensions>,
    mesh_storage: Fetch<'a, AssetStorage<Mesh>>,
    tex_storage: Fetch<'a, AssetStorage<Texture>>,
    ui_image: ReadStorage<'a, UiImage>,
    ui_transform: ReadStorage<'a, UiTransform>,
    unit_mesh: MeshHandle,
    cached_draw_order: &'a mut Vec<(f32, Entity)>,
    supplier: Supplier<'a>,
}

impl<'a> ParallelIterator for DrawUiApply<'a> {
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let DrawUiApply {
            entities,
            screen_dimensions,
            mesh_storage,
            tex_storage,
            ui_image,
            ui_transform,
            unit_mesh,
            cached_draw_order,
            supplier,
            ..
        } = self;

        let entities = &*entities;
        let screen_dimensions = &screen_dimensions;
        let mesh_storage = &mesh_storage;
        let tex_storage = &tex_storage;
        let ui_image = &ui_image;
        let ui_transform = &ui_transform;
        let unit_mesh = &unit_mesh;

        // Populate and update the draw order cache.
        // TODO: Replace all of this with code taking advantage of specs::TrackedStorage.
        // TrackedStorage doesn't exist yet but it will in a later version of specs.
        cached_draw_order.retain(|&(_z, entity)| {
            ui_image.get(entity).is_some() && ui_transform.get(entity).is_some()
        });

        for &mut (ref mut z, entity) in cached_draw_order.iter_mut() {
            *z = ui_transform.get(entity).unwrap().z;
        }

        // Attempt to insert the new entities in sorted position.  Should reduce work during
        // the sorting step.
        for (entity, _image, transform) in (entities, ui_image, ui_transform).join() {
            if cached_draw_order
                .iter()
                .position(|&(_z, cached_entity)| entity == cached_entity)
                .is_none()
            {
                let pos = cached_draw_order
                    .iter()
                    .position(|&(cached_z, _)| transform.z >= cached_z);
                match pos {
                    Some(pos) => cached_draw_order.insert(pos, (transform.z, entity)),
                    None => cached_draw_order.push((transform.z, entity)),
                }
            }
        }

        // Sort from largest z value to smallest z value.
        cached_draw_order.sort_unstable_by(|&(z1, _), &(z2, _)| {
            z2.partial_cmp(&z1).unwrap_or(Ordering::Equal)
        });

        // This pass can't be executed in parallel, so we use a dumby bitset of a
        // single element to provide a fake parallel iterator that performs the entire
        // pass in the first iteration.
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

        let cached_draw_order = &cached_draw_order;

        supplier
            .supply(bitset.par_join().map(move |_id| {
                move |encoder: &mut Encoder, effect: &mut Effect| for &(_z, entity) in
                    cached_draw_order.iter()
                {
                    // These are safe as we guaranteed earlier these entities are present.
                    let ui_transform = ui_transform.get(entity).unwrap();
                    let ui_image = ui_image.get(entity).unwrap();
                    if let Some(mesh) = mesh_storage.get(unit_mesh) {
                        let vbuf = match mesh.buffer(PosTex::ATTRIBUTES) {
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
                        }
                    }
                }
            }))
            .drive_unindexed(consumer)
    }
}

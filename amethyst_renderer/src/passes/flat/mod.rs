use std::ops::Range;

use core::Transform;
use core::cgmath::{Deg, Matrix, Matrix4, SquareMatrix};
use gfx_hal::{Backend, Device};
use gfx_hal::command::{CommandBuffer, RenderPassInlineEncoder};
use gfx_hal::device::ShaderError;
use gfx_hal::memory::Pod;
use gfx_hal::pso::{DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType,
                   DescriptorWrite, EntryPoint, GraphicsShaderSet, Stage, VertexBufferSet};
use gfx_hal::queue::{Supports, Transfer};
use shred::Resources;
use smallvec::SmallVec;
use specs::{Component, DenseVecStorage, Entities, Fetch, Join, ReadStorage, SystemData, World,
            WriteStorage, StorageEntry};

use camera::{ActiveCamera, Camera};
use cirque::Entry;
use descriptors::{DescriptorSet, DescriptorPool, Binder, Layout, Uniform};
use epoch::{CurrentEpoch, Epoch};
use graph::{Data, Pass, PassTag};
use memory::Allocator;
use mesh::{Bind as MeshBind, Mesh};
use uniform::{BasicUniformCache, UniformCache};
use vertex::{PosColor, VertexFormat, VertexFormatted};


#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrProjView {
    transform: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

unsafe impl Pod for TrProjView {}

#[derive(Debug, Default)]
pub struct DrawFlat;
impl<'a, B> Data<'a, B> for DrawFlat
where
    B: Backend,
{
    type PassData = (
        Entities<'a>,

        // Tag
        ReadStorage<'a, PassTag<Self>>,

        // Data
        Fetch<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Mesh<B>>,
        ReadStorage<'a, Transform>,

        // Pass specific components.
        WriteStorage<'a, BasicUniformCache<B, TrProjView>>,
        WriteStorage<'a, DescriptorSet<B, Self>>,
    );
}


impl<B> Pass<B> for DrawFlat
where
    B: Backend,
{
    /// Name of the pass
    const NAME: &'static str = "DrawFlat";

    /// Input attachments
    const INPUTS: usize = 0;

    /// Color attachments
    const COLORS: usize = 1;

    /// Uses depth attachment
    const DEPTH: bool = true;

    /// Uses stencil attachment
    const STENCIL: bool = false;

    /// Vertices format
    const VERTICES: &'static [VertexFormat<'static>] = &[PosColor::VERTEX_FORMAT];

    type Bindings = (Uniform<TrProjView>, ());

    fn layout(layout: Layout<()>) -> Layout<Self::Bindings> {
        layout
            .uniform::<TrProjView, _>(0, Stage::Vertex)
    }

    /// Load shaders
    fn shaders<'a>(
        shaders: &'a mut SmallVec<[B::ShaderModule; 5]>,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>, ShaderError> {
        shaders.clear();
        shaders.push(device.create_shader_module(include_bytes!("vert.spv"))?);
        shaders.push(device.create_shader_module(include_bytes!("frag.spv"))?);

        Ok(GraphicsShaderSet {
            vertex: EntryPoint {
                entry: "main",
                module: &shaders[0],
                specialization: &[],
            },
            hull: None,
            domain: None,
            geometry: None,
            fragment: Some(EntryPoint {
                entry: "main",
                module: &shaders[1],
                specialization: &[],
            }),
        })
    }

    fn prepare<'a, C>(
        &mut self,
        span: Range<Epoch>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        cbuf: &mut CommandBuffer<B, C>,
        (ent, tag, ac, cam, mesh, trs, mut uni, _): <Self as Data<'a, B>>::PassData,
    )
    where
        C: Supports<Transfer>,
    {
        /// Update uniform cache
        for (_, _, tr, ent) in (&tag, mesh.mask(), &trs, &*ent).join() {
            let trprojview = TrProjView {
                transform: (*tr).into(),
                projection: cam.get(ac.entity).unwrap().proj.into(),
                view: (*trs.get(ac.entity).unwrap()).into(),
            };
            match uni.entry(ent).unwrap() {
                StorageEntry::Occupied(mut entry) => {
                    entry.get_mut().update(trprojview, span.clone(), cbuf, allocator, device).unwrap();
                },
                StorageEntry::Vacant(entry) => {
                    entry.insert(BasicUniformCache::new(trprojview, span.clone(), cbuf, allocator, device).unwrap());
                }
            };
        }
    }

    fn draw_inline<'a>(
        &mut self,
        span: Range<Epoch>,
        binder: Binder<B, Self::Bindings>,
        pool: &mut DescriptorPool<B>,
        device: &B::Device,
        mut encoder: RenderPassInlineEncoder<B>,
        (ent, tag, acam, cam, mesh, tr, mut uni, mut descs): <Self as Data<'a, B>>::PassData,
    ) {
        for (_, mesh, uni, e) in (&tag, &mesh, &mut uni, &*ent).join() {
            if  descs.get(e).is_none() {
                descs.insert(e, DescriptorSet::new());
            }

            let desc = descs.get_mut(e).unwrap();
            let set = match desc.get(span.clone()) {
                Entry::Vacant(vacant) => {
                    let mut set = pool.allocate(device);
                    binder.set(&mut set)
                        .uniform(span.clone(), uni)
                        .bind(device);
                    vacant.insert(set)
                }
                Entry::Occupied(occupied) => occupied,
            };

            encoder.bind_graphics_descriptor_sets(binder.layout(), 0, ::std::iter::once(set));

            let mut vertex = VertexBufferSet(vec![]);
            mesh.bind(span.end, <Self as Pass<B>>::VERTICES, &mut vertex)
                .map(|bind| {
                    bind.draw_inline(vertex, &mut encoder);
                })
                .unwrap_or(());
        }
    }

    fn cleanup(&mut self, pool: &mut DescriptorPool<B>, res: &Resources) {}

    fn register(world: &mut World) {
        world.register::<DescriptorSet<B, Self>>();
    }
}

use std::ops::Range;

use core::Transform;
use core::cgmath::{Deg, Matrix, Matrix4, SquareMatrix};
use gfx_hal::{Backend, Device};
use gfx_hal::command::{CommandBuffer, RenderPassInlineEncoder};
use gfx_hal::format::{B8_G8_R8_A8, Bgra8, Depth, Depth32F, Format, Formatted, Rgba8, Srgb, Srgba8};
use gfx_hal::memory::Pod;
use gfx_hal::pso::{DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType,
                   DescriptorWrite, GraphicsShaderSet, ShaderStageFlags, Stage, VertexBufferSet};
use gfx_hal::queue::{Supports, Transfer};
use specs::{Component, DenseVecStorage, Entities, Fetch, Join, ReadStorage, SystemData, World,
            WriteStorage};

use cam::{ActiveCamera, Camera};
use descriptors::{DescriptorSet, DescriptorPool, Binder, Layout, Uniform};
use epoch::{CurrentEpoch, Epoch};
use graph::{Data, Pass, PassTag};
use memory::Allocator;
use mesh::{Bind as MeshBind, Mesh};
use shaders::{GraphicsShaderNameSet, ShaderLoader, ShaderManager};
use uniform::{BasicUniformCache, UniformCache, UniformCacheStorage};
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
    type DrawData = (ReadStorage<'a, Mesh<B>>, ReadStorage<'a, DescriptorSet<B, Self>>);
    type PrepareData = (
        ReadStorage<'a, PassTag<Self>>,
        Entities<'a>,
        Fetch<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Mesh<B>>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, BasicUniformCache<B, TrProjView>>,
        WriteStorage<'a, DescriptorSet<B, Self>>,
    );
}


impl<B> Pass<B> for DrawFlat
where
    B: Backend + ShaderLoader,
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
        manager: &'a mut ShaderManager<B>,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>, ::shaders::Error> {
        manager.load_shader_set(
            GraphicsShaderNameSet::new("flat", false, false, false, true),
            device,
        )
    }

    /// This function designed for
    ///
    /// * allocating buffers and textures
    /// * storing caches in `World`
    /// * filling `DescriptorSet`s
    fn prepare<'a, C>(
        &mut self,
        binder: Binder<Self::Bindings>,
        span: Range<Epoch>,
        pool: &mut DescriptorPool<B>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        (tag, ent, ac, cam, mesh, trs, mut uni, mut desc): <Self as Data<'a, B>>::PrepareData,
    ) where
        C: Supports<Transfer>,
    {
        for (_, _, tr, ent) in (&tag, &mesh.check(), &trs, &*ent).join() {
            uni.update_cache(
                ent,
                TrProjView {
                    transform: Matrix4::identity().into(), // (*tr).into(),
                    projection: cam.get(ac.entity).unwrap().proj.into(),
                    view: (*trs.get(ac.entity).unwrap()).into(),
                },
                span.clone(),
                cbuf,
                allocator,
                device,
            ).unwrap();
        }

        binder.entities(&tag, &uni, &*ent, &mut desc, pool, |binder, uni| {
            binder.uniform(uni).bind(device);
        });
    }

    /// This function designed for
    ///
    /// * binding `DescriptorSet`s
    /// * recording `Transfer` and `Graphics` commands to `CommandBuffer`
    fn draw_inline<'a>(
        &mut self,
        span: Range<Epoch>,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassInlineEncoder<B>,
        (meshes, descs): <Self as Data<'a, B>>::DrawData,
    ) {
        for (ref desc, mesh) in (&descs, &meshes).join() {
            encoder.bind_graphics_descriptor_sets(layout, 0, &[desc.raw()]);

            let mut vertex = VertexBufferSet(vec![]);
            mesh.bind(span.end, &[PosColor::VERTEX_FORMAT], &mut vertex)
                .map(|bind| {
                    bind.draw_inline(vertex, &mut encoder);
                })
                .unwrap_or(());
        }
    }
}

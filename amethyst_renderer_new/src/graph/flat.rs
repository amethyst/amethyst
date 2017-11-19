
use gfx_hal::Backend;
use gfx_hal::command::RawCommandBuffer;
use gfx_hal::format::{Depth32F, Format, Formatted, Rgba8};
use gfx_hal::pso::{DescriptorSetLayoutBinding, VertexBufferSet};
use specs::{Component, DenseVecStorage, Join, ReadStorage, SystemData, World};

use graph::pass::{Data, Pass};
use mesh::{Bind as MeshBind, Mesh};
use vertex::{PosColor, VertexFormat, VertexFormatted};


impl<B> Component for Mesh<B>
where
    B: Backend,
{
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug)]
struct Flat;
impl<'a, B> Data<'a, B> for Flat
where
    B: Backend,
{
    type DrawData = ReadStorage<'a, Mesh<B>>;
    type PrepareData = ();
}


impl<B> Pass<B> for Flat
where
    B: Backend,
{
    /// Name of the pass
    const NAME: &'static str = "Flat";

    /// Input attachments format
    const INPUTS: &'static [Format] = &[];

    /// Color attachments format
    const COLORS: &'static [Format] = &[Rgba8::SELF];

    /// DepthStencil attachment format
    const DEPTH_STENCIL: Option<Format> = Some(Depth32F::SELF);

    /// Bindings
    const BINDINGS: &'static [DescriptorSetLayoutBinding] = &[];

    /// Vertices format
    const VERTICES: &'static [VertexFormat<'static>] = &[PosColor::VERTEX_FORMAT];

    fn new() -> Self {
        Flat
    }

    /// This function designed for
    ///
    /// * allocating buffers and textures
    /// * storing caches in `World`
    /// * filling `DescriptorSet`s
    fn prepare<'a>(
        &mut self,
        cbuf: &mut B::CommandBuffer,
        layout: &B::PipelineLayout,
        device: &mut B::Device,
        data: (),
    ) {
    }

    /// This function designed for
    ///
    /// * binding `DescriptorSet`s
    /// * recording `Transfer` and `Graphics` commands to `CommandBuffer`
    fn draw<'a>(&mut self, cbuf: &mut B::CommandBuffer, meshes: ReadStorage<'a, Mesh<B>>) {
        for mesh in meshes.join() {
            let mut vertex = VertexBufferSet(vec![]);
            mesh.bind(&[PosColor::VERTEX_FORMAT], &mut vertex).map(
                |bind| {
                    bind.draw(vertex, cbuf);
                },
            ).unwrap_or(());
        }
    }
}

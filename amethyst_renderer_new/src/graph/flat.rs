use core::Transform;
use gfx_hal::Backend;
use gfx_hal::command::{CommandBuffer, RenderPassInlineEncoder};
use gfx_hal::format::{B8_G8_R8_A8, Bgra8, Depth32F, Format, Formatted, Rgba8, Srgb, Srgba8};
use gfx_hal::pso::{DescriptorSetLayoutBinding, GraphicsShaderSet, Stage, VertexBufferSet};
use specs::{Fetch, Join, ReadStorage, SystemData, World, WriteStorage};

use cam::{ActiveCamera, Camera};
use epoch::Epoch;
use graph::pass::{Data, Pass};
use mesh::{Bind as MeshBind, Mesh};
use shaders::{GraphicsShaderNameSet, ShaderLoader, ShaderManager};
use uniform::{IntoUniform, UniformCache};
use vertex::{PosColor, VertexFormat, VertexFormatted};

type Sbgra8 = (B8_G8_R8_A8, Srgb);

#[derive(Debug, Default)]
pub struct Flat;
impl<'a, B> Data<'a, B> for Flat
where
    B: Backend,
{
    type DrawData = (
        //Fetch<'a, ActiveCamera>,
        //ReadStorage<'a, UniformCache<B, Camera>>,
        //ReadStorage<'a, UniformCache<B, Transform>>,
        ReadStorage<'a, Mesh<B>>,
    );
    type PrepareData = (
        //Fetch<'a, ActiveCamera>,
        //ReadStorage<'a, Camera>,
        //WriteStorage<'a, UniformCache<B, Camera>>,
        //ReadStorage<'a, Transform>,
        //WriteStorage<'a, UniformCache<B, Transform>>,
    );
}


impl<B> Pass<B> for Flat
where
    B: Backend + ShaderLoader,
{
    /// Name of the pass
    const NAME: &'static str = "Flat";

    /// Input attachments format
    const INPUTS: &'static [Format] = &[];

    /// Color attachments format
    const COLORS: &'static [Format] = &[Srgba8::SELF];

    /// DepthStencil attachment format
    const DEPTH_STENCIL: Option<Format> = None;

    /// Bindings
    const BINDINGS: &'static [DescriptorSetLayoutBinding] = &[];

    /// Vertices format
    const VERTICES: &'static [VertexFormat<'static>] = &[PosColor::VERTEX_FORMAT];

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
        finish: Epoch,
        cbuf: &mut CommandBuffer<B, C>,
        layout: &B::PipelineLayout,
        device: &B::Device,
        data: <Self as Data<'a, B>>::PrepareData,
    ) {
    }

    /// This function designed for
    ///
    /// * binding `DescriptorSet`s
    /// * recording `Transfer` and `Graphics` commands to `CommandBuffer`
    fn draw_inline<'a>(
        &mut self,
        finish: Epoch,
        mut encoder: RenderPassInlineEncoder<B>,
        (meshes,): <Self as Data<'a, B>>::DrawData,
    ) {
        for mesh in meshes.join() {
            let mut vertex = VertexBufferSet(vec![]);
            mesh.bind(finish, &[PosColor::VERTEX_FORMAT], &mut vertex)
                .map(|bind| {
                    bind.draw_inline(vertex, &mut encoder);
                })
                .unwrap_or(());
        }
    }
}

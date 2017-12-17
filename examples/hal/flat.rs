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
use descriptors::Descriptors;
use epoch::{CurrentEpoch, Epoch};
use graph::{Data, Pass};
use memory::Allocator;
use mesh::{Bind as MeshBind, Mesh};
use shaders::{GraphicsShaderNameSet, ShaderLoader, ShaderManager};
use uniform::{BasicUniformCache, UniformCache, UniformCacheStorage};
use vertex::{PosColor, VertexFormat, VertexFormatted};

type Sbgra8 = (B8_G8_R8_A8, Srgb);

pub struct Desc<B: Backend>(B::DescriptorSet);

impl<B> Component for Desc<B>
where
    B: Backend,
{
    type Storage = DenseVecStorage<Desc<B>>;
}

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
    type DrawData = (ReadStorage<'a, Mesh<B>>, ReadStorage<'a, Desc<B>>);
    type PrepareData = (
        Entities<'a>,
        Fetch<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Mesh<B>>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, BasicUniformCache<B, TrProjView>>,
        WriteStorage<'a, Desc<B>>,
    );
}


impl<B> Pass<B> for DrawFlat
where
    B: Backend + ShaderLoader,
{
    /// Name of the pass
    const NAME: &'static str = "DrawFlat";

    /// Input attachments format
    const INPUTS: &'static [Format] = &[];

    /// Color attachments format
    const COLORS: &'static [Format] = &[Srgba8::SELF];

    /// DepthStencil attachment format
    const DEPTH_STENCIL: Option<Format> = Some(Depth32F::SELF);

    /// Bindings
    const BINDINGS: &'static [DescriptorSetLayoutBinding] = &[
        DescriptorSetLayoutBinding {
            binding: 0,
            ty: DescriptorType::UniformBuffer,
            count: 1,
            stage_flags: ShaderStageFlags::VERTEX,
        },
    ];

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
        current: &CurrentEpoch,
        descriptors: &mut Descriptors<B>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        (ent, ac, cam, mesh, trs, mut uni, mut desc): <Self as Data<'a, B>>::PrepareData,
    ) where
        C: Supports<Transfer>,
    {
        for (ent, _, tr) in (&*ent, &mesh.check(), &trs).join() {
            uni.update_cache(
                ent,
                TrProjView {
                    transform: Matrix4::identity().into(), // (*tr).into(),
                    projection: cam.get(ac.entity).unwrap().proj.into(),
                    view: (*trs.get(ac.entity).unwrap()).into(),
                },
                finish,
                current,
                cbuf,
                allocator,
                device,
            ).unwrap();
        }

        for (ent, _, _, uni) in (&*ent, &mesh.check(), &trs, &uni).join() {
            if desc.get(ent).is_none() {
                let set = descriptors.get();
                {
                    let (buf, range) = uni.get_cached();
                    let write = DescriptorSetWrite {
                        set: &set,
                        binding: 0,
                        array_offset: 0,
                        write: DescriptorWrite::UniformBuffer(vec![(buf.raw(), range)]),
                    };
                    device.update_descriptor_sets(&[write]);
                }
                desc.insert(ent, Desc(set));
            }
        }
    }

    /// This function designed for
    ///
    /// * binding `DescriptorSet`s
    /// * recording `Transfer` and `Graphics` commands to `CommandBuffer`
    fn draw_inline<'a>(
        &mut self,
        finish: Epoch,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassInlineEncoder<B>,
        (meshes, descs): <Self as Data<'a, B>>::DrawData,
    ) {
        for (&Desc(ref desc), mesh) in (&descs, &meshes).join() {
            encoder.bind_graphics_descriptor_sets(layout, 0, &[desc]);

            let mut vertex = VertexBufferSet(vec![]);
            mesh.bind(finish, &[PosColor::VERTEX_FORMAT], &mut vertex)
                .map(|bind| {
                    bind.draw_inline(vertex, &mut encoder);
                })
                .unwrap_or(());
        }
    }
}

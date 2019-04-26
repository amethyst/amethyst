use crate::{
    pod::UiViewArgs,
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal::{self, device::Device, pso::Descriptor, Backend},
        memory::Write as _,
        resource::{
            Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle,
        },
    },
    submodules::gather::CameraGatherer,
    util,
};
use amethyst_core::ecs::Resources;

#[derive(Debug)]
pub struct UiEnvironmentSub<B: Backend> {
    layout: RendyHandle<DescriptorSetLayout<B>>,
    per_image: Vec<PerImageUiEnvironmentSub<B>>,
}

#[derive(Debug)]
struct PerImageUiEnvironmentSub<B: Backend> {
    buffer: Escape<Buffer<B>>,
    set: Escape<DescriptorSet<B>>,
}

impl<B: Backend> UiEnvironmentSub<B> {
    pub fn new(factory: &Factory<B>) -> Result<Self, failure::Error> {
        Ok(Self {
            layout: set_layout! {factory, 1 UniformBuffer VERTEX},
            per_image: Vec::new(),
        })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    pub fn process(&mut self, factory: &Factory<B>, index: usize, res: &Resources) {
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image
                    .push(PerImageUiEnvironmentSub::new(factory, &self.layout));
            }
            &mut self.per_image[index]
        };
        this_image.process(factory, res)
    }

    #[inline]
    pub fn bind(
        &self,
        index: usize,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        self.per_image[index].bind(pipeline_layout, set_id, encoder);
    }
}

impl<B: Backend> PerImageUiEnvironmentSub<B> {
    fn new(factory: &Factory<B>, layout: &RendyHandle<DescriptorSetLayout<B>>) -> Self {
        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: util::align_size::<UiViewArgs>(1, 1),
                    usage: hal::buffer::Usage::UNIFORM,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        let set = factory.create_descriptor_set(layout.clone()).unwrap();
        let descriptor = Descriptor::Buffer(buffer.raw(), None..None);
        unsafe {
            factory.write_descriptor_sets(Some(util::desc_write(set.raw(), 0, descriptor)));
        }
        Self { buffer, set }
    }

    #[inline]
    fn bind(
        &self,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        encoder.bind_graphics_descriptor_sets(
            pipeline_layout,
            set_id,
            Some(self.set.raw()),
            std::iter::empty(),
        );
    }

    fn process(&mut self, factory: &Factory<B>, res: &Resources) {
        // TODO(happens): This should happen in build
        // let args_size = util::align_size::<UiViewArgs>(1, 1);
        // let inverted_screen_size = <(ScreenDimensions)>::fetch()

        // let mut mapped = self.buffer.map(factory, 0..args_size).unwrap();
        // let mut writer = unsafe { mapped.write::<u8>(factory, 0..args_size).unwrap() };
        // let dst_slice = unsafe { writer.slice() };

        // dst_slice.copy_from_slice(util::slice_as_bytes(&[projview]));
    }
}

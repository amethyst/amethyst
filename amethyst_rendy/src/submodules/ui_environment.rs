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
    util,
};
use glsl_layout::AsStd140;

#[derive(Debug)]
pub struct UiEnvironmentSub<B: Backend> {
    layout: RendyHandle<DescriptorSetLayout<B>>,
    buffer: Escape<Buffer<B>>,
    set: Escape<DescriptorSet<B>>,
}

impl<B: Backend> UiEnvironmentSub<B> {
    pub fn new(factory: &Factory<B>) -> Result<Self, failure::Error> {
        let layout: RendyHandle<DescriptorSetLayout<B>> =
            set_layout! {factory, 1 UniformBuffer VERTEX};

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

        Ok(Self { layout, buffer, set })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    pub fn setup(&mut self, factory: &Factory<B>, framebuffer_size: (u32, u32)) {
        let args_size = util::align_size::<UiViewArgs>(1, 1);
        let (width, height) = framebuffer_size;
        let inverse_window_size = [
            1. / width as f32,
            1. / height as f32,
        ].into();

        let args = UiViewArgs { inverse_window_size }.std140();
        let mut mapped = self.buffer.map(factory, 0..args_size).unwrap();
        let mut writer = unsafe { mapped.write::<u8>(factory, 0..args_size).unwrap() };
        let dst_slice = unsafe { writer.slice() };
        dst_slice.copy_from_slice(util::slice_as_bytes(&[args]));
    }

    #[inline]
    pub fn bind(
        &self,
        index: usize,
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
}

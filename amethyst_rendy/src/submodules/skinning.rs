use crate::{
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal::{self, device::Device, pso::Descriptor},
        memory::Write as _,
        resource::{Buffer, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    },
    skinning::JointTransforms,
    types::Backend,
    util,
};
use fnv::FnvHashMap;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Debug)]
pub struct SkinningSub<B: Backend> {
    layout: RendyHandle<DescriptorSetLayout<B>>,
    skin_offset_map: FnvHashMap<u32, u32>,
    staging: Vec<[[f32; 4]; 4]>,
    per_image: Vec<PerImageSkinningSub<B>>,
}

#[derive(Debug)]
struct PerImageSkinningSub<B: Backend> {
    buffer: Option<Escape<Buffer<B>>>,
    set: Escape<DescriptorSet<B>>,
}

impl<B: Backend> SkinningSub<B> {
    pub fn new(factory: &Factory<B>) -> Result<Self, failure::Error> {
        Ok(Self {
            layout: set_layout! {factory, [1] StorageBuffer VERTEX},
            skin_offset_map: Default::default(),
            staging: Vec::new(),
            per_image: Vec::new(),
        })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    pub fn commit(&mut self, factory: &Factory<B>, index: usize) {
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image
                    .push(PerImageSkinningSub::new(factory, &self.layout));
            }
            &mut self.per_image[index]
        };
        this_image.commit(factory, util::slice_as_bytes(&self.staging));
        self.staging.clear();
        self.skin_offset_map.clear();
    }

    pub fn insert(&mut self, joints: &JointTransforms) -> u32 {
        #[cfg(feature = "profiler")]
        profile_scope!("insert");

        let staging = &mut self.staging;
        *self
            .skin_offset_map
            .entry(joints.skin.id())
            .or_insert_with(|| {
                let len = staging.len();
                staging.extend(
                    joints
                        .matrices
                        .iter()
                        .map(|m| -> [[f32; 4]; 4] { (*m).into() }),
                );
                len as u32
            })
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

impl<B: Backend> PerImageSkinningSub<B> {
    fn new(factory: &Factory<B>, layout: &RendyHandle<DescriptorSetLayout<B>>) -> Self {
        Self {
            buffer: None,
            set: factory.create_descriptor_set(layout.clone()).unwrap(),
        }
    }

    fn commit(&mut self, factory: &Factory<B>, data: &[u8]) {
        if data.len() == 0 {
            return;
        }

        let allocated = util::ensure_buffer(
            &factory,
            &mut self.buffer,
            hal::buffer::Usage::STORAGE,
            rendy::memory::Dynamic,
            data.len() as u64,
        )
        .unwrap();

        if let Some(buffer) = self.buffer.as_mut() {
            if allocated {
                unsafe {
                    factory.write_descriptor_sets(Some(util::desc_write(
                        self.set.raw(),
                        0,
                        Descriptor::Buffer(buffer.raw(), Some(0)..None),
                    )));
                }
            }

            let mut mapped = buffer.map(factory.device(), 0..data.len() as u64).unwrap();
            let mut writer = unsafe {
                mapped
                    .write(factory.device(), 0..data.len() as u64)
                    .unwrap()
            };
            let dst_slice = unsafe { writer.slice() };
            dst_slice.copy_from_slice(data);
        }
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
}

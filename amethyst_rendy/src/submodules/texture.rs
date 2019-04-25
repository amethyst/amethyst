use crate::{
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal::{device::Device, Backend},
        resource::{DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    },
    types::Texture,
    util,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::ecs::{Read, Resources, SystemData};

#[derive(Debug)]
enum TextureState<B: Backend> {
    Unloaded {
        generation: u32,
    },
    Loaded {
        set: Escape<DescriptorSet<B>>,
        generation: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

#[derive(Debug)]
pub struct TextureSub<B: Backend> {
    generation: u32,
    layout: RendyHandle<DescriptorSetLayout<B>>,
    lookup: util::LookupBuilder<u32>,
    textures: Vec<TextureState<B>>,
}

impl<B: Backend> TextureSub<B> {
    pub fn new(factory: &Factory<B>) -> Result<Self, failure::Error> {
        Ok(Self {
            layout: set_layout! {factory, 1 CombinedImageSampler FRAGMENT},
            lookup: util::LookupBuilder::new(),
            textures: Vec::with_capacity(1024),
            generation: 0,
        })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    pub fn maintain(&mut self) {
        self.generation += self.generation.wrapping_add(1);
    }

    fn try_insert(
        &mut self,
        factory: &Factory<B>,
        res: &Resources,
        handle: &Handle<Texture<B>>,
    ) -> Option<TextureState<B>> {
        use util::{desc_write, texture_desc};
        let tex_storage = <(Read<'_, AssetStorage<Texture<B>>>)>::fetch(res);

        let tex = tex_storage.get(handle)?;
        let set = factory.create_descriptor_set(self.layout.clone()).unwrap();
        unsafe {
            let set = set.raw();
            factory.write_descriptor_sets(vec![desc_write(set, 0, texture_desc(tex))]);
        }
        Some(TextureState::Loaded {
            set,
            generation: self.generation,
        })
    }

    pub fn insert(
        &mut self,
        factory: &Factory<B>,
        res: &Resources,
        handle: &Handle<Texture<B>>,
    ) -> Option<(TextureId, bool)> {
        let id = self.lookup.forward(handle.id());
        match self.textures.get_mut(id) {
            Some(TextureState::Loaded { .. }) => {
                return Some((TextureId(id as u32), false));
            }
            Some(TextureState::Unloaded { generation }) if *generation == self.generation => {
                return None
            }
            _ => {}
        };

        let (new_state, loaded) = self
            .try_insert(factory, res, handle)
            .map(|s| (s, true))
            .unwrap_or_else(|| {
                (
                    TextureState::Unloaded {
                        generation: self.generation,
                    },
                    false,
                )
            });

        if self.textures.len() == id {
            self.textures.push(new_state);
        } else {
            self.textures[id] = new_state;
        }

        if loaded {
            Some((TextureId(id as u32), true))
        } else {
            None
        }
    }

    #[inline]
    pub fn bind(
        &self,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        texture_id: TextureId,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) -> bool {
        match &self.textures[texture_id.0 as usize] {
            TextureState::Loaded { set, .. } => {
                encoder.bind_graphics_descriptor_sets(
                    pipeline_layout,
                    set_id,
                    Some(set.raw()),
                    std::iter::empty(),
                );
                true
            }
            _ => false,
        }
    }
}

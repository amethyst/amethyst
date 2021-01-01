//! Texture submodule for per-image submission.
use amethyst_assets::{AssetHandle, AssetStorage, Handle, LoadHandle, WeakHandle};
use amethyst_core::ecs::*;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal::{self, device::Device},
        resource::{DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    },
    types::{Backend, Texture},
    util,
};

#[derive(Debug)]
enum TextureState<B: Backend> {
    Unloaded {
        generation: u32,
    },
    Loaded {
        set: Escape<DescriptorSet<B>>,
        generation: u32,
        version: u32,
        handle: WeakHandle,
        layout: hal::image::Layout,
    },
}

/// Texture ID newtype, preventing users from creating arbitrary `TextureId`. Represented as a `u32`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

/// Texture helper submodule for allocating and binding textures and abstracting per-image submissions.
#[derive(Debug)]
pub struct TextureSub<B: Backend> {
    generation: u32,
    layout: RendyHandle<DescriptorSetLayout<B>>,
    lookup: util::LookupBuilder<LoadHandle>,
    textures: Vec<TextureState<B>>,
}

impl<B: Backend> TextureSub<B> {
    /// Create a new Texture for submission, allocated using the provided `Factory`
    pub fn new(factory: &Factory<B>) -> Result<Self, hal::pso::CreationError> {
        use rendy::hal::pso::*;

        let layout = factory
            .create_descriptor_set_layout(util::set_layout_bindings(vec![(
                1,
                DescriptorType::Image {
                    ty: ImageDescriptorType::Sampled { with_sampler: true },
                },
                ShaderStageFlags::FRAGMENT,
            )]))?
            .into();

        Ok(Self {
            layout,
            lookup: util::LookupBuilder::new(),
            textures: Vec::with_capacity(1024),
            generation: 0,
        })
    }

    /// Returns the raw `DescriptorSetLayout` for a Texture
    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    /// Generationally track our currently allocated vs. used textures and release memory for any
    /// textures which have been removed from this submission set.
    pub fn maintain(&mut self, factory: &Factory<B>, resources: &Resources) {
        #[cfg(feature = "profiler")]
        profile_scope!("maintain");

        use util::{desc_write, texture_desc};
        let tex_storage = resources.get::<AssetStorage<Texture>>().unwrap();
        for state in self.textures.iter_mut() {
            match state {
                TextureState::Loaded {
                    generation,
                    set,
                    version,
                    handle,
                    layout,
                } if *generation == self.generation => {
                    if let Some((new_tex, new_version)) = tex_storage.get_asset_with_version(handle)
                    {
                        if *version != new_version {
                            if let Some(desc) = texture_desc(new_tex, *layout) {
                                unsafe {
                                    let set = set.raw();
                                    factory.write_descriptor_sets(vec![desc_write(set, 0, desc)]);
                                }
                                *version = new_version;
                            } else {
                                *state = TextureState::Unloaded {
                                    generation: self.generation,
                                };
                            }
                        }
                    } else {
                        *state = TextureState::Unloaded {
                            generation: self.generation,
                        };
                    }
                }
                _ => {}
            }
        }
        self.generation = self.generation.wrapping_add(1);
    }

    /// Try to insert a new texture for submission in this texture batch. Returns None if it fails.
    fn try_insert(
        &mut self,
        factory: &Factory<B>,
        resources: &Resources,
        handle: &Handle<Texture>,
        layout: hal::image::Layout,
    ) -> Option<TextureState<B>> {
        #[cfg(feature = "profiler")]
        profile_scope!("try_insert");

        use util::{desc_write, texture_desc};
        let tex_storage = resources.get::<AssetStorage<Texture>>().unwrap();

        let (tex, version) = tex_storage.get_asset_with_version(handle)?;
        let desc = texture_desc(tex, layout)?;
        let set = factory.create_descriptor_set(self.layout.clone()).unwrap();
        unsafe {
            let set = set.raw();
            factory.write_descriptor_sets(vec![desc_write(set, 0, desc)]);
        }
        Some(TextureState::Loaded {
            set,
            generation: self.generation,
            version,
            handle: handle.downgrade(),
            layout,
        })
    }

    /// Try to insert a new texture for submission in this texture batch.
    pub fn insert(
        &mut self,
        factory: &Factory<B>,
        resources: &Resources,
        handle: &Handle<Texture>,
        layout: hal::image::Layout,
    ) -> Option<(TextureId, bool)> {
        #[cfg(feature = "profiler")]
        profile_scope!("insert");

        let id = self.lookup.forward(handle.load_handle());
        match self.textures.get(id) {
            // If handle is dead, new texture was loaded (handle id is reused)
            // FIXME figure out if we still need the is_dead() guard 
            Some(TextureState::Loaded { handle, .. }) /* if !handle.is_dead() */ => {
                return Some((TextureId(id as u32), false));
            }
            Some(TextureState::Unloaded { generation }) if *generation == self.generation => {
                return None
            }
            _ => {}
        };

        let (new_state, loaded) = self
            .try_insert(factory, resources, handle, layout)
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

    /// Returns true of the supplied `TextureId` is already loaded.
    #[inline]
    pub fn loaded(&self, texture_id: TextureId) -> bool {
        matches!(
            &self.textures[texture_id.0 as usize],
            TextureState::Loaded { handle, .. }
        )
    }

    /// Bind all textures
    #[inline]
    pub fn bind(
        &self,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        texture_id: TextureId,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        match &self.textures[texture_id.0 as usize] {
            TextureState::Loaded { set, .. } => unsafe {
                encoder.bind_graphics_descriptor_sets(
                    pipeline_layout,
                    set_id,
                    Some(set.raw()),
                    std::iter::empty(),
                );
            },
            _ => panic!("Trying to bind unloaded texture"),
        }
    }
}

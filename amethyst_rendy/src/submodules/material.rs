//! Material abstraction submodule.
use crate::{
    mtl::{Material, StaticTextureSet},
    pod,
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal::{self, adapter::PhysicalDevice, device::Device, pso::Descriptor},
        memory::Write as _,
        resource::{
            Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle,
        },
    },
    types::{Backend, Texture},
    util,
};
use amethyst_assets::{AssetStorage, Handle, WeakHandle};
use amethyst_core::ecs::*;
use glsl_layout::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Debug)]
struct SlotAllocator {
    vaccants: Vec<u64>,
    lowest_vaccant_idx: usize,
    alloc_step: usize,
}

impl SlotAllocator {
    pub fn new(block_size: usize) -> Self {
        Self {
            alloc_step: (block_size + 63) / 64,
            vaccants: vec![0; (block_size + 63) / 64],
            lowest_vaccant_idx: 0,
        }
    }

    pub fn would_overflow(&self) -> bool {
        self.lowest_vaccant_idx == self.vaccants.len()
    }

    pub fn reserve(&mut self) -> usize {
        if let Some((i, vaccant)) = self.vaccants[self.lowest_vaccant_idx..]
            .iter_mut()
            .enumerate()
            .find(|(_, vaccant)| **vaccant != std::u64::MAX)
        {
            let vaccant_idx = self.lowest_vaccant_idx + i;
            let free_subid = (!*vaccant).trailing_zeros();
            *vaccant |= 1 << free_subid;
            self.lowest_vaccant_idx = if *vaccant == std::u64::MAX {
                vaccant_idx + 1
            } else {
                vaccant_idx
            };

            vaccant_idx * 64 + free_subid as usize
        } else {
            let vaccant_idx = self.vaccants.len();
            self.lowest_vaccant_idx = vaccant_idx;
            self.vaccants.resize(vaccant_idx + self.alloc_step, 0);
            self.vaccants[self.lowest_vaccant_idx] = 1;
            vaccant_idx * 64
        }
    }

    pub fn release(&mut self, index: usize) {
        self.lowest_vaccant_idx = self.lowest_vaccant_idx.min(index / 64);
        self.vaccants[index / 64] &= !(1 << (index % 64));
    }
}

#[derive(Debug)]
struct SlottedBuffer<B: Backend> {
    buffer: Escape<Buffer<B>>,
    elem_size: u64,
}

impl<B: Backend> SlottedBuffer<B> {
    fn new(
        factory: &Factory<B>,
        elem_size: u64,
        capacity: usize,
        usage: hal::buffer::Usage,
    ) -> Result<Self, failure::Error> {
        Ok(Self {
            buffer: factory.create_buffer(
                BufferInfo {
                    size: elem_size * (capacity as u64),
                    usage,
                },
                rendy::memory::Dynamic,
            )?,
            elem_size,
        })
    }

    pub fn descriptor(&self, id: usize) -> Descriptor<'_, B> {
        let offset = (id as u64) * self.elem_size;
        Descriptor::Buffer(
            self.buffer.raw(),
            Some(offset)..Some(offset + self.elem_size),
        )
    }

    fn write(&mut self, factory: &Factory<B>, id: usize, data: &[u8]) {
        let offset = self.elem_size * id as u64;
        let mut mapped = self
            .buffer
            .map(factory.device(), offset..offset + data.len() as u64)
            .unwrap();
        unsafe {
            let mut writer = mapped
                .write(factory.device(), 0..data.len() as u64)
                .unwrap();
            writer.write(data);
        }
    }
}

#[derive(Debug)]
enum MaterialState<B: Backend> {
    Unloaded {
        generation: u32,
    },
    Loaded {
        set: Escape<DescriptorSet<B>>,
        slot: usize,
        generation: u32,
        handle: WeakHandle<Material>,
    },
}

/// Material ID newtype, preventing users from creating arbitrary `MaterialId`. Represented as a `u32`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(u32);

/// Material helper submodule for allocating and binding materials and their associated textures.
#[derive(Debug)]
pub struct MaterialSub<B: Backend, T: for<'a> StaticTextureSet<'a>> {
    generation: u32,
    layout: RendyHandle<DescriptorSetLayout<B>>,
    lookup: util::LookupBuilder<u32>,
    allocator: SlotAllocator,
    buffers: Vec<SlottedBuffer<B>>,
    materials: Vec<MaterialState<B>>,
    marker: std::marker::PhantomData<T>,
}

impl<B: Backend, T: for<'a> StaticTextureSet<'a>> MaterialSub<B, T> {
    /// Create a new `MaterialSub` using the provided rendy `Factory`
    pub fn new(factory: &Factory<B>) -> Result<Self, failure::Error> {
        Ok(Self {
            layout: set_layout! {
                factory,
                [1] UniformBuffer hal::pso::ShaderStageFlags::FRAGMENT,
                [T::len()] CombinedImageSampler hal::pso::ShaderStageFlags::FRAGMENT
            },
            lookup: util::LookupBuilder::new(),
            allocator: SlotAllocator::new(1024),
            buffers: vec![Self::create_buffer(factory)?],
            materials: Vec::with_capacity(1024),
            generation: 0,
            marker: std::marker::PhantomData,
        })
    }

    fn create_buffer(factory: &Factory<B>) -> Result<SlottedBuffer<B>, failure::Error> {
        let align = factory
            .physical()
            .limits()
            .min_uniform_buffer_offset_alignment;
        let material_step = util::align_size::<pod::Material>(align, 1);
        SlottedBuffer::new(factory, material_step, 1024, hal::buffer::Usage::UNIFORM)
    }

    /// Returns the raw `DescriptorSetLayout` for this environment
    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    /// Increment the internal generation counter.
    pub fn maintain(&mut self) {
        self.generation += self.generation.wrapping_add(1);
    }

    /// Releases any materials not used in the current generation.
    fn collect_unused(&mut self) {
        let cur_generation = self.generation;
        // let allocator = &mut self.allocator;
        for material in self.materials.iter_mut().filter(|m| match m {
            MaterialState::Loaded { generation, .. } => *generation < cur_generation,
            _ => false,
        }) {
            if let MaterialState::Loaded { slot, .. } = material {
                self.allocator.release(*slot);
            }
            *material = MaterialState::Unloaded {
                generation: self.generation.wrapping_sub(1),
            }
        }
    }

    /// Attempts to insert a new material to this collection.
    fn try_insert(
        &mut self,
        factory: &Factory<B>,
        resources: &Resources,
        handle: &Handle<Material>,
    ) -> Option<MaterialState<B>> {
        #[cfg(feature = "profiler")]
        profile_scope!("try_insert");

        use util::{desc_write, slice_as_bytes, texture_desc};

        let mat_storage = resources.get::<AssetStorage<Material>>().unwrap();
        let tex_storage = resources.get::<AssetStorage<Texture>>().unwrap();

        let mat = mat_storage.get(handle)?;

        let has_tex = T::textures(mat).any(|t| {
            !tex_storage
                .get(t)
                .map_or(false, |tex| B::unwrap_texture(tex).is_some())
        });
        if has_tex {
            return None;
        }

        let pod = pod::Material::from_material(&mat).std140();

        if self.allocator.would_overflow() {
            self.collect_unused();
        }

        let slot = self.allocator.reserve();
        let buf_num = slot / 1024;
        let buf_slot = slot % 1024;

        while self.buffers.len() <= buf_num {
            self.buffers.push(Self::create_buffer(factory).unwrap());
        }
        self.buffers[buf_num].write(factory, buf_slot, slice_as_bytes(&[pod]));
        let set = factory.create_descriptor_set(self.layout.clone()).unwrap();
        let buf_desc = self.buffers[buf_num].descriptor(buf_slot);

        unsafe {
            let set = set.raw();

            let tex_descs = T::textures(mat).enumerate().map(|(i, t)| {
                desc_write(
                    set,
                    (i + 1) as u32,
                    texture_desc(
                        tex_storage.get(t).unwrap(),
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )
                    .unwrap(),
                )
            });

            let desc_iter = std::iter::once(desc_write(set, 0, buf_desc)).chain(tex_descs);
            factory.write_descriptor_sets(desc_iter);
        }
        Some(MaterialState::Loaded {
            set,
            slot,
            generation: self.generation,
            handle: handle.downgrade(),
        })
    }

    /// Inserts a new material to this collection.
    pub fn insert(
        &mut self,
        factory: &Factory<B>,
        resources: &Resources,
        handle: &Handle<Material>,
    ) -> Option<(MaterialId, bool)> {
        #[cfg(feature = "profiler")]
        profile_scope!("insert");

        let id = self.lookup.forward(handle.id());
        match self.materials.get_mut(id) {
            Some(MaterialState::Loaded {
                slot,
                generation,
                handle,
                ..
            }) => {
                // If handle is dead, new material was loaded (handle id reused)
                if handle.is_dead() {
                    self.allocator.release(*slot);
                } else {
                    // Material loaded and ready
                    *generation = self.generation;
                    return Some((MaterialId(id as u32), false));
                }
            }
            Some(MaterialState::Unloaded { generation }) if *generation == self.generation => {
                return None
            }
            _ => {}
        };

        debug_assert!(self.materials.len() >= id);
        let (new_state, loaded) = self
            .try_insert(factory, resources, handle)
            .map(|s| (s, true))
            .unwrap_or_else(|| {
                (
                    MaterialState::Unloaded {
                        generation: self.generation,
                    },
                    false,
                )
            });

        if self.materials.len() == id {
            self.materials.push(new_state);
        } else {
            self.materials[id] = new_state;
        }

        if loaded {
            Some((MaterialId(id as u32), true))
        } else {
            None
        }
    }

    /// Returns `true` if the supplied `MaterialId` is already loaded.
    #[inline]
    pub fn loaded(&self, material_id: MaterialId) -> bool {
        match &self.materials[material_id.0 as usize] {
            MaterialState::Loaded { .. } => true,
            _ => false,
        }
    }

    /// Binds all material descriptor sets and textures contained in this collection.
    #[inline]
    pub fn bind(
        &self,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        material_id: MaterialId,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        match &self.materials[material_id.0 as usize] {
            MaterialState::Loaded { set, .. } => unsafe {
                encoder.bind_graphics_descriptor_sets(
                    pipeline_layout,
                    set_id,
                    Some(set.raw()),
                    std::iter::empty(),
                );
            },
            _ => panic!("Trying to bind unloaded material"),
        };
    }
}

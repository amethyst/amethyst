use crate::{
    batch::{BatchData, BatchPrimitives, GroupIterator},
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    pass::util,
    pod::{IntoPod, SpriteArgs, ViewArgs},
    sprite::{SpriteRender, SpriteSheet},
    sprite_visibility::SpriteVisibility,
    types::Texture,
};
use amethyst_assets::AssetStorage;
use amethyst_core::{
    ecs::{Join, Read, ReadStorage, Resources, SystemData},
    math::Vector4,
    transform::GlobalTransform,
};
use derivative::Derivative;
use fnv::FnvHashMap;
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{
            Layout, PrepareResult, SetLayout, SimpleGraphicsPipeline, SimpleGraphicsPipelineDesc,
        },
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{
        adapter::PhysicalDevice,
        buffer::Usage as BufferUsage,
        device::Device,
        format::Format,
        pso::{
            BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc, Descriptor,
            DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType, ElemStride, Element,
            EntryPoint, GraphicsShaderSet, InstanceRate, ShaderStageFlags, Specialization,
        },
        Backend,
    },
    memory::Write,
    mesh::AsVertex,
    resource::{Buffer, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    shader::Shader,
};
use smallvec::SmallVec;

const TEXTURE_ARRAY_SIZE: usize = 32;

/// Draw sprites without lighting
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawFlat2DDesc {
    transparency: Option<(ColorBlendDesc, Option<DepthStencilDesc>)>,
}

impl DrawFlat2DDesc {
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        color: ColorBlendDesc,
        depth: Option<DepthStencilDesc>,
    ) -> Self {
        log::trace!("Transparency set: {:?}, {:?}", color, depth);
        self.transparency = Some((color, depth));
        self
    }
}

impl<B: Backend> SimpleGraphicsPipelineDesc<B, Resources> for DrawFlat2DDesc {
    type Pipeline = DrawFlat2D<B>;

    fn load_shader_set<'a>(
        &self,
        storage: &'a mut Vec<B::ShaderModule>,
        factory: &mut Factory<B>,
        _aux: &Resources,
    ) -> GraphicsShaderSet<'a, B> {
        storage.clear();

        log::trace!("Loading shader module '{:#?}'", *super::SPRITE_VERTEX);
        storage.push(unsafe { super::SPRITE_VERTEX.module(factory).unwrap() });

        log::trace!("Loading shader module '{:#?}'", *super::SPRITE_FRAGMENT);
        storage.push(unsafe { super::SPRITE_FRAGMENT.module(factory).unwrap() });

        GraphicsShaderSet {
            vertex: EntryPoint {
                entry: "main",
                module: &storage[0],
                specialization: Specialization::default(),
            },
            fragment: Some(EntryPoint {
                entry: "main",
                module: &storage[1],
                specialization: Specialization::default(),
            }),
            hull: None,
            domain: None,
            geometry: None,
        }
    }

    fn colors(&self) -> Vec<ColorBlendDesc> {
        if let Some((color, _)) = self.transparency {
            vec![color]
        } else {
            vec![ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA)]
        }
    }

    fn depth_stencil(&self) -> Option<DepthStencilDesc> {
        if let Some((_, stencil)) = self.transparency {
            stencil
        } else {
            None
        }
    }

    fn vertices(&self) -> Vec<(Vec<Element<Format>>, ElemStride, InstanceRate)> {
        vec![SpriteArgs::VERTEX.gfx_vertex_input_desc(1)]
    }

    fn layout(&self) -> Layout {
        Layout {
            sets: vec![
                SetLayout {
                    bindings: vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: ShaderStageFlags::GRAPHICS,
                        immutable_samplers: false,
                    }],
                },
                SetLayout {
                    bindings: vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::CombinedImageSampler,
                        count: TEXTURE_ARRAY_SIZE,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    }],
                },
            ],
            push_constants: vec![(ShaderStageFlags::FRAGMENT, 0..1)],
        }
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _resource: &Resources,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        _set_layouts: &[RendyHandle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, failure::Error> {
        let ubo_offset_align = factory
            .physical()
            .limits()
            .min_uniform_buffer_offset_alignment;

        Ok(DrawFlat2D {
            per_image: Vec::with_capacity(4),
            sprite_data: Default::default(),
            ubo_offset_align,
            total_instances: 0,
            ..Default::default()
        })
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawFlat2D<B: Backend> {
    per_image: Vec<PerImage<B>>,
    sprite_data: FnvHashMap<u32, SpriteData>,
    ubo_offset_align: u64,
    total_instances: u64,
}

impl<B: Backend> DrawFlat2D<B> {
    #[inline]
    fn desc_write<'a>(
        set: &'a B::DescriptorSet,
        binding: u32,
        array_offset: usize,
        descriptor: Descriptor<'a, B>,
    ) -> DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>> {
        DescriptorSetWrite {
            set,
            binding,
            array_offset,
            descriptors: Some(descriptor),
        }
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
struct PerImage<B: Backend> {
    projview_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    tex_id_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    sprites_buf: Option<Escape<Buffer<B>>>,
    projview_set: Option<Escape<DescriptorSet<B>>>,
    textures_set: Vec<Escape<DescriptorSet<B>>>,
}

// Outer layer - texture_num / TEXTURE_ARRAY_SIZE (batch id)
// Inner layer - texture_num % TEXTURE_ARRAY_SIZE (texture id in batch)
// data        - SpriteArgs sequence
type SpriteBatchData = BatchData<u32, SmallVec<[SpriteArgs; 1]>>;

#[derive(Debug)]
struct SpriteData {
    sprites: Vec<SpriteBatchData>,
}

struct BatchSprite;

impl BatchPrimitives for BatchSprite {
    type Shell = SpriteData;
    type Batch = SpriteBatchData;

    fn wrap_batch(batch: Self::Batch) -> Self::Shell {
        let mut sprites = Vec::with_capacity(1024);
        sprites.push(batch);
        SpriteData { sprites }
    }
    fn push(shell: &mut Self::Shell, batch: Self::Batch) {
        shell.sprites.push(batch);
    }
    fn batches_mut(shell: &mut Self::Shell) -> &mut [Self::Batch] {
        &mut shell.sprites
    }
}

impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawFlat2D<B> {
    type Desc = DrawFlat2DDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        set_layouts: &[RendyHandle<DescriptorSetLayout<B>>],
        index: usize,
        resources: &Resources,
    ) -> PrepareResult {
        let (
            active_camera,
            cameras,
            sprite_sheet_storage,
            tex_storage,
            visibilities,
            hiddens,
            hidden_props,
            sprite_renders,
            global_transforms,
        ) = <(
            Option<Read<'_, ActiveCamera>>,
            ReadStorage<'_, Camera>,
            Read<'_, AssetStorage<SpriteSheet<B>>>,
            Read<'_, AssetStorage<Texture<B>>>,
            Option<Read<'_, SpriteVisibility>>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, SpriteRender<B>>,
            ReadStorage<'_, GlobalTransform>,
        ) as SystemData>::fetch(resources);

        // ensure resources for this image are available
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image.push(PerImage::default());
            }
            &mut self.per_image[index]
        };

        let (_, projview) = util::prepare_camera(&active_camera, &cameras, &global_transforms);

        // Write the projview buffer and set.
        let projview_size = util::align_size::<ViewArgs>(self.ubo_offset_align, 1);
        if util::ensure_buffer(
            factory,
            &mut this_image.projview_buffer,
            BufferUsage::UNIFORM,
            rendy::memory::Dynamic,
            projview_size,
        )
        .unwrap()
        {
            let projview_set = this_image.projview_set.get_or_insert_with(|| {
                factory
                    .create_descriptor_set(set_layouts[0].clone())
                    .unwrap()
            });

            let desc_projview = Descriptor::Buffer(
                this_image.projview_buffer.as_ref().unwrap().raw(),
                Some(0)..Some(projview_size),
            );

            unsafe {
                factory.write_descriptor_sets(Some(Self::desc_write(
                    projview_set.raw(),
                    0,
                    0,
                    desc_projview,
                )));
            }
        }

        if let Some(buffer) = this_image.projview_buffer.as_mut() {
            unsafe {
                factory
                    .upload_visible_buffer(buffer, 0, &[projview])
                    .unwrap();
            }
        }

        let mut tex_lookup = util::LookupBuilder::new();
        let sprite_data_ref = &mut self.sprite_data;
        let mut total_instances = 0;

        for (_, data) in sprite_data_ref.iter_mut() {
            data.sprites.clear();
        }

        match visibilities {
            None => {
                (
                    &sprite_renders,
                    &global_transforms,
                    !&hiddens,
                    !&hidden_props,
                )
                    .join()
                    .map(|(sprite_render, global, _, _)| {
                        log::trace!("Add sprite");
                        let sprite_sheet = sprite_sheet_storage
                                .get(&sprite_render.sprite_sheet)
                                .expect(
                                    "Unreachable: Existence of sprite sheet checked when collecting the sprites",
                                );

                        let tex_id = tex_lookup.forward(sprite_sheet.texture.id()) as u32;
                        let sprite = &sprite_sheet.sprites[sprite_render.sprite_number];

                        let transform = &global.0;
                        let dir_x = transform.column(0) * sprite.width;
                        let dir_y = transform.column(1) * sprite.height;
                        let pos = transform
                            * Vector4::new(-sprite.offsets[0], -sprite.offsets[1], 0.0, 1.0);

                        let batch_data = SpriteArgs {
                            dir_x: dir_x.xy().into_pod(),
                            dir_y: dir_y.xy().into_pod(),
                            pos: pos.xy().into_pod(),
                            u_offset: [sprite.tex_coords.left, sprite.tex_coords.right]
                                .into(),
                            v_offset: [sprite.tex_coords.bottom, sprite.tex_coords.top]
                                .into(),
                            depth: pos.z,
                        };

                        (tex_id, batch_data)
                    })
                    .filter(|(tex_id, _)| tex_storage.contains_id(*tex_id))
                    .for_each_group(|tex_id, batch_data| {
                        let tex_pk = tex_id / TEXTURE_ARRAY_SIZE as u32;
                        let tex_sk = tex_id % TEXTURE_ARRAY_SIZE as u32;
                        total_instances += batch_data.len() as u64;
                        BatchSprite::insert_batch(sprite_data_ref.entry(tex_pk), tex_sk, batch_data);
                    });
            }
            Some(ref visibility) => {
                for (_sprite_render, _global, _) in (
                    &sprite_renders,
                    &global_transforms,
                    &visibility.visible_unordered,
                )
                    .join()
                {
                    unimplemented!();
                }

                for entity in &visibility.visible_ordered {
                    //let screen = screens.contains(*entity);
                    if let Some(_sprite_render) = sprite_renders.get(*entity) {
                        unimplemented!();
                    }
                }
            }
        }

        sprite_data_ref.retain(|_, data| data.sprites.len() > 0);

        let num_chunks =
            (tex_lookup.backward().len() + TEXTURE_ARRAY_SIZE - 1) / TEXTURE_ARRAY_SIZE;
        if this_image.textures_set.len() < num_chunks {
            this_image.textures_set.resize_with(num_chunks, || {
                factory
                    .create_descriptor_set(set_layouts[1].clone())
                    .unwrap()
            });
        }

        {
            let tex_storage = &tex_storage;

            let writes_iter = tex_lookup
                .backward()
                .chunks(TEXTURE_ARRAY_SIZE)
                .zip(this_image.textures_set.iter())
                .map(|(tex_ids, set)| {
                    let expand = TEXTURE_ARRAY_SIZE - tex_ids.len();
                    let ids_iter = tex_ids
                        .iter()
                        .chain(std::iter::repeat(&tex_ids[0]).take(expand));

                    let descriptors = ids_iter.map(move |tex_id| {
                        // Validated by `filter` in batch collection
                        debug_assert!(tex_storage.contains_id(*tex_id));
                        let Texture(tex) = unsafe { tex_storage.get_by_id_unchecked(*tex_id) };
                        Descriptor::CombinedImageSampler(
                            tex.view().raw(),
                            rendy::hal::image::Layout::ShaderReadOnlyOptimal,
                            tex.sampler().raw(),
                        )
                    });
                    DescriptorSetWrite {
                        set: set.raw(),
                        binding: 0,
                        array_offset: 0,
                        descriptors,
                    }
                });

            unsafe {
                factory.write_descriptor_sets(writes_iter);
            }
        }

        let sprite_args_size = total_instances * std::mem::size_of::<SpriteArgs>() as u64;
        util::ensure_buffer(
            factory,
            &mut this_image.sprites_buf,
            BufferUsage::VERTEX,
            rendy::memory::Dynamic,
            sprite_args_size,
        )
        .unwrap();

        if let Some(buffer) = this_image.sprites_buf.as_mut() {
            unsafe {
                let mut mapped = buffer.map(factory, 0..sprite_args_size).unwrap();
                let mut writer = mapped.write(factory, 0..sprite_args_size).unwrap();
                let dst_slice = writer.slice();

                let mut offset = 0;
                for (_, sprite_data) in sprite_data_ref {
                    for batch in &sprite_data.sprites {
                        let bytes = util::slice_as_bytes(&batch.collection);
                        dst_slice[offset..].copy_from_slice(bytes);
                        offset += bytes.len();
                    }
                }
            }
        }

        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _resources: &Resources,
    ) {
        let this_image = &self.per_image[index];

        if this_image.sprites_buf.is_none() {
            return;
        }

        let projview_set = this_image.projview_set.as_ref().unwrap().raw();
        let sprites_buf = this_image.sprites_buf.as_ref().unwrap().raw();

        encoder.bind_graphics_descriptor_sets(layout, 0, Some(projview_set), None);
        encoder.bind_vertex_buffers(0, Some((sprites_buf, 0)));

        let mut offset = 0;
        for (tex_pk, data) in self.sprite_data.iter() {
            let tex_set = this_image.textures_set[*tex_pk as usize].raw();
            encoder.bind_graphics_descriptor_sets(layout, 1, Some(tex_set), None);

            for BatchData { key, collection } in &data.sprites {
                let num_instances = collection.len() as u32;
                encoder.push_constants(layout, ShaderStageFlags::FRAGMENT, 0, &[*key]);
                encoder.draw(0..6, offset..offset + num_instances);
                offset += num_instances;
            }
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {}
}

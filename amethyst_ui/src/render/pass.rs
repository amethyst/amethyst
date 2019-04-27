use std::cmp::Ordering;
use derivative::Derivative;
use crate::{
    batch::OrderedOneLevelBatch,
    hidden::{Hidden, HiddenPropagate},
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    pod::{UiArgs},
    submodules::{DynamicVertex, UiEnvironmentSub, TextureId, TextureSub},
    types::Texture, resources::Tint,
    util,
};
use amethyst_assets::Handle;
use amethyst_core::ecs::prelude::{
    Join, ReadStorage, Resources, SystemData,
    WriteStorage, Entities, Entity,
};
use hibitset::BitSet;
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        BufferAccess, GraphContext, ImageAccess, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device, pso, Backend},
    mesh::AsVertex,
    shader::Shader,
};
use std::borrow::Borrow;

use amethyst_ui::{
    Selected, UiText, TextEditing, UiTransform,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawUiDesc;

impl DrawUiDesc {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, Resources> for DrawUiDesc {
    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &Resources,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, Resources>>, failure::Error> {
        let mut env = UiEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertex::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        // NOTE(happens): The only thing the uniform depends on is the
        // framebuffer size. Build is called whenever this changes, so
        // we only need to call this once at this point.
        env.setup(factory, (framebuffer_width, framebuffer_height));

        Ok(Box::new(DrawUi::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            change: Default::default(),
            cached_draw_order: Default::default(),
            images: Default::default(),

            // cached_color_textures: HashMap::new(),
        }))
    }
}

#[derive(Debug)]
pub struct DrawUi<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: UiEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertex<B, UiArgs>,
    images: OrderedOneLevelBatch<TextureId, UiArgs>,
    change: util::ChangeDetection,

    cached_draw_order: CachedDrawOrder,

    // TODO(happens): Can we just use the TextureId here? We need
    // `Debug` on this and `Hash` on palette::Srgba
    // cached_color_textures: HashMap<palette::Srgba, Texture<B>>,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Default(bound = ""))]
struct CachedDrawOrder {
    pub cached: BitSet,
    pub cache: Vec<(f32, Entity)>,
}

impl<B: Backend> RenderGroup<B, Resources> for DrawUi<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        resources: &Resources,
    ) -> PrepareResult {
        let (
            entities,
            textures,
            transforms,
            mut texts,
            text_editings,
            hiddens,
            hidden_propagates,
            selected,
            tints,
        ) = <(
            Entities<'_>,
            ReadStorage<'_, Handle<Texture<B>>>,
            ReadStorage<'_, UiTransform>,
            WriteStorage<'_, UiText>,
            ReadStorage<'_, TextEditing>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, Selected>,
            ReadStorage<'_, Tint>,
        ) as SystemData>::fetch(resources);

        self.textures.maintain();
        self.images.swap_clear();
        let mut changed = false;

        let images_ref = &mut self.images;
        let textures_ref = &mut self.textures;

        // Populate and update the draw order cache.
        let bitset = &mut self.cached_draw_order.cached;
        self.cached_draw_order.cache.retain(|&(_z, entity)| {
            let keep = transforms.contains(entity);
            if !keep {
                bitset.remove(entity.id());
            }
            keep
        });

        for &mut (ref mut z, entity) in &mut self.cached_draw_order.cache {
            *z = transforms
                .get(entity)
                .expect("Unreachable: Enities are collected from a cache of prepopulate entities")
                .global_z();
        }

        // Attempt to insert the new entities in sorted position. Should reduce work during
        // the sorting step.
        let transform_set = transforms.mask().clone();

        // Create a bitset containing only the new indices.
        let new = (&transform_set ^ &self.cached_draw_order.cached) & &transform_set;
        for (entity, transform, _new) in (&*entities, &transforms, &new).join() {
            let pos = self
                .cached_draw_order
                .cache
                .iter()
                .position(|&(cached_z, _)| transform.global_z() >= cached_z);

            match pos {
                Some(pos) => self
                    .cached_draw_order
                    .cache
                    .insert(pos, (transform.global_z(), entity)),
                None => self
                    .cached_draw_order
                    .cache
                    .push((transform.global_z(), entity)),
            }
        }

        self.cached_draw_order.cached = transform_set;

        // Sort from largest z value to smallest z value.
        // Most of the time this shouldn't do anything but you still need it
        // for if the z values change.
        self.cached_draw_order
            .cache
            .sort_unstable_by(|&(z1, _), &(z2, _)| {
                z1.partial_cmp(&z2).unwrap_or(Ordering::Equal)
            });

        let highest_abs_z = (&transforms,).join()
            .map(|t| t.0.global_z())
            // TODO(happens): Use max_by here?
            .fold(1.0, |highest, current| current.abs().max(highest));

        for &(_z, entity) in &self.cached_draw_order.cache {
            // Skip hidden entities
            if hiddens.contains(entity) || hidden_propagates.contains(entity) {
                continue;
            }

            let transform = transforms
                .get(entity)
                .expect("Unreachable: Entity is guaranteed to be present based on earlier actions");

            let tint: [f32; 4] = tints
                .get(entity)
                .cloned()
                .unwrap_or(Tint(palette::Srgba::new(1., 1., 1., 1.)))
                .into();

            if let Some(texture) = textures.get(entity) {
                let args = UiArgs {
                    coords: [transform.pixel_x(), transform.pixel_y()].into(),
                    dimensions: [transform.pixel_width(), transform.pixel_height()].into(),
                    color: tint.into(),
                };

                if let Some((tex_id, this_changed)) =
                    textures_ref.insert(factory, resources, texture) {
                    changed = changed || this_changed;
                    images_ref.insert(tex_id, Some(args));
                }
            }

            // TODO(happens): Text drawing
            // if let Some(text) = texts.get_mut(entity) {}
        }

        changed = changed || self.images.changed();
        self.vertex.write(factory, index, self.images.count() as u64, Some(self.images.data()));
        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _resources: &Resources,
    ) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);
        self.vertex.bind(index, 0, &mut encoder);
        for (&tex, range) in self.images.iter() {
            self.textures.bind(layout, 1, tex, &mut encoder);
            encoder.draw(0..6, range);
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &Resources) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_sprite_pipeline<B: Backend, I>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    transparent: bool,
    layouts: I,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error>
where
    I: IntoIterator,
    I::Item: Borrow<B::DescriptorSetLayout>,
{
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let shader_vertex = unsafe { super::UI_VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { super::UI_FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(UiArgs::VERTEX, 1)])
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc(
                    pso::ColorMask::ALL,
                    pso::BlendState::ALPHA,
                )]),
        )
        .build(factory, None);

    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}

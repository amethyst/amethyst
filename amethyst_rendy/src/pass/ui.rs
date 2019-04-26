use std::{collections::HashMap, cmp::Ordering};
use derivative::Derivative;
use crate::{
    batch::{GroupIterator, OneLevelBatch, OrderedOneLevelBatch},
    hidden::{Hidden, HiddenPropagate},
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    pod::{UiArgs, UiViewArgs},
    submodules::{DynamicVertex, UiEnvironmentSub, TextureId, TextureSub},
    types::Texture, resources::Tint,
    util,
};
use amethyst_assets::{AssetStorage, Handle, Loader};
use amethyst_core::ecs::prelude::{
    Join, Read, ReadExpect, ReadStorage, Resources, SystemData,
    WriteStorage, Entities, Entity,
};
use hibitset::BitSet;
use amethyst_window::ScreenDimensions;
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
    fn buffers(&self) -> Vec<BufferAccess> {
        vec![]
    }

    fn images(&self) -> Vec<ImageAccess> {
        vec![]
    }

    fn depth(&self) -> bool {
        true
    }

    fn colors(&self) -> usize {
        1
    }

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
        let env = UiEnvironmentSub::new(factory)?;
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

        // TODO(happens): Set uniform
        let invert_window_size = [
            1. / framebuffer_width as f32,
            1. / framebuffer_height as f32,
        ];

        Ok(Box::new(DrawUi::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            change: Default::default(),
            cached_draw_order: Default::default(),

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
    // sprites: OrderedOneLevelBatch<TextureId, SpriteArgs>,
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
            loader,
            screen_dimensions,
            texture_storage,
            // font_assets_storage,
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
            ReadExpect<'_, Loader>,
            ReadExpect<'_, ScreenDimensions>,
            Read<'_, AssetStorage<Texture<B>>>,
            // Read<'_, AssetStorage<FontAsset>>,
            ReadStorage<'_, Handle<Texture<B>>>,
            ReadStorage<'_, UiTransform>,
            WriteStorage<'_, UiText>,
            ReadStorage<'_, TextEditing>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, Selected>,
            ReadStorage<'_, Tint>,
        ) as SystemData>::fetch(resources);

        self.env.process(factory, index, resources);
        self.textures.maintain();
        let mut changed = false;

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

        // TODO(happens): Do we keep drawing this using an instanced mesh?
        // let mesh = self
        //     .mesh
        //     .as_ref()
        //     .expect("`DrawUi::compile` was not called before `DrawUi::apply`");

        // let vbuf = match mesh.buffer(PosTex::ATTRIBUTES) {
        //     Some(vbuf) => vbuf.clone(),
        //     None => return,
        // };
        // effect.data.vertex_bufs.push(vbuf);

        let highest_abs_z = (&transforms,).join()
            .map(|t| t.0.global_z())
            // TODO(happens): Use max_by here?
            .fold(1.0, |highest, current| current.abs().max(highest));

        for &(_z, entity) in &self.cached_draw_order.cache {
            // Skip hidden entities
            if hiddens.contains(entity) || hidden_propagates.contains(entity) {
                continue;
            }

            let ui_transform = transforms
                .get(entity)
                .expect("Unreachable: Entity is guaranteed to be present based on earlier actions");

            let tint: [f32; 4] = tints
                .get(entity)
                .cloned()
                .unwrap_or(Tint(palette::Srgba::new(1., 1., 1., 1.)))
                .into();

            if let Some(tex) = textures
                .get(entity)
                .and_then(|tex| texture_storage.get(&tex))
            {
                // TODO(happens): Draw texture with params:
                // [ui_transform.pixel_x, ui_transform.pixel_y] -> coords
                // [ui_transform.pixel_width, ui_transform.pixel_height] -> dimensions
                // tint -> color

                // Why were these uniforms before?!

                // TODO(happens): We were binding the texture anew every time here
                // before, which is pretty inefficient. Can we batch these and
                // draw them similarly to before, so at least the color textures
                // are not rebound all the time?
            }

            // TODO(happens): Text drawing
            if let Some(text) = texts.get_mut(entity) {}
        }

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _resources: &Resources,
    ) {
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);
        self.vertex.bind(index, 0, &mut encoder);

        // TODO(happens): Draw instances
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
                    if transparent {
                        pso::BlendState::ALPHA
                    } else {
                        pso::BlendState::Off
                    },
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

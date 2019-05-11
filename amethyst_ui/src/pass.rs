use crate::{Selected, TextEditing, UiText, UiTransform};
use amethyst_assets::{AssetStorage, Handle, Loader};
use amethyst_core::{
    ecs::{
        Entities, Entity, Join, Read, ReadExpect, ReadStorage, Resources, SystemData, WriteStorage,
    },
    Hidden, HiddenPropagate,
};
use amethyst_rendy::{
    batch::OrderedOneLevelBatch,
    palette,
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    rendy::{
        command::{QueueId, RenderPassEncoder},
        factory::Factory,
        graph::{
            render::{PrepareResult, RenderGroup, RenderGroupDesc},
            GraphContext, NodeBuffer, NodeImage,
        },
        hal::{self, device::Device, format::Format, pso},
        mesh::{AsVertex, VertexFormat},
        shader::{Shader, ShaderKind, SourceLanguage, SpirvShader, StaticShaderInfo},
        texture::palette::load_from_srgb,
    },
    resources::Tint,
    simple_shader_set,
    submodules::{DynamicUniform, DynamicVertex, TextureId, TextureSub},
    types::{Backend, Texture},
    ChangeDetection,
};
use derivative::Derivative;
use glsl_layout::{vec2, vec4, AsStd140};
use hibitset::BitSet;
use std::cmp::Ordering;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
struct UiArgs {
    coords: vec2,
    dimensions: vec2,
    color: vec4,
}

impl AsVertex for UiArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Float, "coords"),
            (Format::Rg32Float, "dimensions"),
            (Format::Rgba32Float, "color"),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
struct UiViewArgs {
    inverse_window_size: vec2,
}

lazy_static::lazy_static! {
    static ref UI_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/ui.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref UI_FRAGMENT: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/ui.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();
}

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
        resources: &Resources,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, Resources>>, failure::Error> {
        let mut env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertex::new();

        let (pipeline, pipeline_layout) = build_ui_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        // NOTE(happens): The only thing the uniform depends on is the
        // framebuffer size. Build is called whenever this changes, so
        // we only need to call this once at this point.
        // This also means that all frames are using index 0 during drawing.
        let view_args = UiViewArgs {
            inverse_window_size: [
                1.0 / framebuffer_width as f32,
                1.0 / framebuffer_height as f32,
            ]
            .into(),
        };
        env.write(factory, 0, view_args.std140());

        let (loader, tex_storage) =
            <(ReadExpect<'_, Loader>, Read<'_, AssetStorage<Texture>>)>::fetch(resources);
        let white_tex = loader.load_from_data(
            load_from_srgb(palette::named::WHITE).into(),
            (),
            &tex_storage,
        );

        Ok(Box::new(DrawUi::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            change: Default::default(),
            cached_draw_order: Default::default(),
            images: Default::default(),
            white_tex,
        }))
    }
}

#[derive(Debug)]
pub struct DrawUi<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, UiViewArgs>,
    textures: TextureSub<B>,
    vertex: DynamicVertex<B, UiArgs>,
    images: OrderedOneLevelBatch<TextureId, UiArgs>,
    change: ChangeDetection,
    cached_draw_order: CachedDrawOrder,
    white_tex: Handle<Texture>,
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
            ReadStorage<'_, Handle<Texture>>,
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
            .sort_unstable_by(|&(z1, _), &(z2, _)| z1.partial_cmp(&z2).unwrap_or(Ordering::Equal));

        let highest_abs_z = (&transforms,)
            .join()
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

            // let (texture, tint) = match (textures.get(entity), tints.get(entity)) {
            //     (Some(tex), tint) => {

            //     }
            //     (None, tint) => {

            //     }
            //     (None, None) => {
            //         (
            //             self.white_tex,
            //             Tint(palette::Srgba::new(1., 1., 1., 1.))
            //         )
            //     }
            // }

            let tint: [f32; 4] = tints
                .get(entity)
                .cloned()
                .unwrap_or(Tint(palette::Srgba::new(1., 1., 1., 1.)))
                .into();

            if let Some(texture) = textures.get(entity) {
                let args = UiArgs {
                    coords: [transform.pixel_x(), transform.pixel_y()].into(),
                    dimensions: [transform.pixel_width, transform.pixel_height].into(),
                    color: tint.into(),
                };

                if let Some((tex_id, this_changed)) =
                    textures_ref.insert(factory, resources, texture)
                {
                    changed = changed || this_changed;
                    images_ref.insert(tex_id, Some(args));
                }
            }

            // TODO(happens): Text drawing
            // if let Some(text) = texts.get_mut(entity) {}
        }

        changed = changed || self.images.changed();
        self.vertex.write(
            factory,
            index,
            self.images.count() as u64,
            Some(self.images.data()),
        );
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
        // use index 0 unconditionally. This is written only once.
        self.env.bind(0, &self.pipeline_layout, 0, &mut encoder);
        self.vertex.bind(index, 0, &mut encoder);
        for (&tex, range) in self.images.iter() {
            dbg!(&range);
            self.textures.bind(layout, 1, tex, &mut encoder);
            encoder.draw(0..4, range);
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

fn build_ui_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let shader_vertex = unsafe { UI_VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { UI_FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(UiArgs::vertex(), 1)])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
                .with_shaders(simple_shader_set(&shader_vertex, Some(&shader_fragment)))
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

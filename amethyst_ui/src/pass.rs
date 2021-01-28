use std::{cmp::Ordering, collections::HashSet};

use amethyst_assets::{AssetStorage, DefaultLoader, Handle, Loader, ProcessingQueue};
use amethyst_core::{ecs::*, Hidden, HiddenPropagate};
use amethyst_error::Error;
use amethyst_rendy::{
    batch::OrderedOneLevelBatch,
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    palette,
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    rendy::{
        command::{QueueId, RenderPassEncoder},
        factory::Factory,
        graph::{
            render::{PrepareResult, RenderGroup, RenderGroupDesc},
            GraphContext, NodeBuffer, NodeImage,
        },
        hal::{
            self,
            device::Device,
            format::Format,
            pso::{self, ShaderStageFlags},
        },
        mesh::{AsVertex, VertexFormat},
        shader::{Shader, SpirvShader},
        texture::palette::load_from_srgba,
    },
    resources::Tint,
    simple_shader_set,
    sprite::Sprites,
    submodules::{DynamicUniform, DynamicVertexBuffer, TextureId, TextureSub},
    system::GraphAuxData,
    types::{Backend, Texture, TextureData},
    ChangeDetection, SpriteSheet,
};
use amethyst_window::ScreenDimensions;
use derivative::Derivative;
use glsl_layout::{vec2, vec4, Uniform};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    glyphs::{UiGlyphs, UiGlyphsResource},
    Selected, TextEditing, UiImage, UiTransform,
};

/// A [RenderPlugin] for rendering UI elements.
#[derive(Debug, Default)]
pub struct RenderUi {
    target: Target,
}

impl RenderUi {
    /// Select render target on which UI should be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderUi {
    fn on_build<'a, 'b>(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.insert(UiGlyphsResource::new(resources));

        builder.add_system(crate::glyphs::UiGlyphsSystem::<B>::default());
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
        _resources: &Resources,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Overlay, DrawUiDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }

    fn should_rebuild(&mut self, _world: &World, _resources: &Resources) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Uniform)]
#[repr(C, align(4))]
pub(crate) struct UiArgs {
    pub(crate) coords: vec2,
    pub(crate) dimensions: vec2,
    pub(crate) tex_coord_bounds: vec4,
    pub(crate) color: vec4,
    pub(crate) color_bias: vec4,
}

impl AsVertex for UiArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "coords"),
            (Format::Rg32Sfloat, "dimensions"),
            (Format::Rgba32Sfloat, "tex_coord_bounds"),
            (Format::Rgba32Sfloat, "color"),
            (Format::Rgba32Sfloat, "color_bias"),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Uniform)]
struct UiViewArgs {
    inverse_window_size: vec2,
}

lazy_static::lazy_static! {
    static ref UI_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../compiled/ui.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref UI_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../compiled/ui.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();
}

/// A UI drawing pass that draws UI elements and text in screen-space
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawUiDesc;

impl DrawUiDesc {
    /// Create new DrawUI pass description
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, GraphAuxData> for DrawUiDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        data: &GraphAuxData,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, GraphAuxData>>, pso::CreationError> {
        #[cfg(feature = "profiler")]
        profile_scope!("build");

        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_ui_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        let (loader, tex_storage) = (
            data.resources.get::<DefaultLoader>().unwrap(),
            data.resources
                .get::<ProcessingQueue<TextureData>>()
                .unwrap(),
        );
        let white_tex = loader.load_from_data(
            load_from_srgba(palette::Srgba::new(1., 1., 1., 1.)).into(),
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
            batches: Default::default(),
            white_tex,
        }))
    }
}

/// A UI drawing pass that draws UI elements and text in screen-space
#[derive(Debug)]
pub struct DrawUi<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, UiViewArgs>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, UiArgs>,
    batches: OrderedOneLevelBatch<TextureId, UiArgs>,
    change: ChangeDetection,
    cached_draw_order: CachedDrawOrder,
    white_tex: Handle<Texture>,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Default(bound = ""))]
struct CachedDrawOrder {
    pub cached: HashSet<Entity>,
    pub cache: Vec<(f32, Entity)>,
}

impl<B: Backend> RenderGroup<B, GraphAuxData> for DrawUi<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        aux: &GraphAuxData,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare");
        let GraphAuxData { world, resources } = aux;

        let glyphs_res = resources.get::<UiGlyphsResource>().unwrap();
        let screen_dimensions = resources.get::<ScreenDimensions>().unwrap();

        self.batches.swap_clear();
        let mut changed = false;

        let (white_tex_id, glyph_tex_id) = {
            if let (Some((white_tex_id, white_changed)), Some((glyph_tex_id, glyph_changed))) = (
                self.textures.insert(
                    factory,
                    resources,
                    &self.white_tex,
                    hal::image::Layout::ShaderReadOnlyOptimal,
                ),
                glyphs_res.glyph_tex().and_then(|tex| {
                    self.textures
                        .insert(factory, resources, tex, hal::image::Layout::General)
                }),
            ) {
                changed = changed || white_changed || glyph_changed;
                (white_tex_id, glyph_tex_id)
            } else {
                // Internal texture was not loaded. This can happen only during the
                // first frame ever, as the texture ref never changes and is loaded by
                // assets processor immediately. Having this check here allows to skip
                // many branches in actual drawing code below.
                // `DrawReuse` is OK here because we are sure that nothing gets drawn anyway.
                self.textures.maintain(factory, resources);
                return PrepareResult::DrawReuse;
            }
        };

        let mut query_transforms = <(Entity, &UiTransform)>::query();

        let mut to_remove = HashSet::new();

        self.cached_draw_order.cache.retain(|(_z, entity)| {
            let keep = query_transforms.get(*world, *entity).is_ok();
            if !keep {
                to_remove.insert(*entity);
            }
            keep
        });

        to_remove.iter().for_each(|e| {
            self.cached_draw_order.cached.remove(e);
        });

        for &mut (ref mut z, entity) in &mut self.cached_draw_order.cache {
            *z = query_transforms
                .get(*world, entity)
                .expect("Unreachable: Enities are collected from a cache of prepopulate entities")
                .1
                .global_z();
        }

        // Attempt to insert the new entities in sorted position. Should reduce work during
        // the sorting step.
        {
            query_transforms.for_each(*world, |(entity, transform)| {
                // We only want the new ones.
                // The old way (pre legion) to do it was with bitset :
                // let new = (&transform_set ^ &cache.cached) & &transform_set;
                if !self.cached_draw_order.cached.contains(entity) {
                    let pos = self
                        .cached_draw_order
                        .cache
                        .iter()
                        .position(|&(cached_z, _)| transform.global_z() >= cached_z);
                    match pos {
                        Some(pos) => {
                            self.cached_draw_order
                                .cache
                                .insert(pos, (transform.global_z(), *entity))
                        }
                        None => {
                            self.cached_draw_order
                                .cache
                                .push((transform.global_z(), *entity))
                        }
                    }
                }
            });
            self.cached_draw_order.cached.clear();
            query_transforms.for_each(*world, |(entity, _)| {
                self.cached_draw_order.cached.insert(*entity);
            });
        }

        // Sort from largest z value to smallest z value.
        // Most of the time this shouldn't do anything but you still need it
        // for if the z values change.
        self.cached_draw_order
            .cache
            .sort_unstable_by(|&(z1, _), &(z2, _)| z1.partial_cmp(&z2).unwrap_or(Ordering::Equal));

        let mut query = <(
            &UiTransform,
            Option<&Tint>,
            Option<&UiImage>,
            Option<&UiGlyphs>,
            Option<&Selected>,
            Option<&TextEditing>,
        )>::query()
        .filter(!component::<Hidden>() & !component::<HiddenPropagate>());

        for &(_z, entity) in &self.cached_draw_order.cache {
            let (
                transform,
                maybe_tint,
                maybe_image,
                maybe_glyph,
                maybe_selected,
                maybe_txt_editing,
            ) = query
                .get(*world, entity)
                .expect("Unreachable: Entity is guaranteed to be present based on earlier actions");

            let tint = maybe_tint.map(|t| {
                let (r, g, b, a) = t.0.into_components();
                [r, g, b, a]
            });

            if let Some(image) = maybe_image {
                let this_changed = render_image(
                    factory,
                    aux,
                    transform,
                    image,
                    &tint,
                    white_tex_id,
                    &mut self.textures,
                    &mut self.batches,
                );
                changed = changed || this_changed;
            };

            if let Some(glyph_data) = maybe_glyph {
                if !glyph_data.sel_vertices.is_empty() {
                    self.batches
                        .insert(white_tex_id, glyph_data.sel_vertices.iter().cloned());
                }

                // blinking cursor
                if maybe_selected.is_some() {
                    if let Some(editing) = maybe_txt_editing {
                        let blink_on = editing.cursor_blink_timer < 0.25;
                        let (w, h) = match (blink_on, editing.use_block_cursor) {
                            // use degenerate quad, but still insert so batches will not change
                            (false, false) => (0., 0.),
                            (true, false) => (2., glyph_data.height),
                            (false, true) => {
                                (glyph_data.space_width, 1.0f32.max(glyph_data.height * 0.1))
                            }
                            (true, true) => (glyph_data.space_width, glyph_data.height),
                        };
                        // align to baseline
                        let base_x = glyph_data.cursor_pos.0 + w * 0.5;
                        let base_y = glyph_data.cursor_pos.1 - (glyph_data.height - h) * 0.5;

                        let min_x = transform.pixel_x + transform.pixel_width * -0.5;
                        let max_x = transform.pixel_x + transform.pixel_width * 0.5;
                        let min_y = transform.pixel_y + transform.pixel_height * -0.5;
                        let max_y = transform.pixel_y + transform.pixel_height * 0.5;

                        let left = (base_x - w * 0.5).max(min_x).min(max_x);
                        let right = (base_x + w * 0.5).max(min_x).min(max_x);
                        let top = (base_y - h * 0.5).max(min_y).min(max_y);
                        let bottom = (base_y + h * 0.5).max(min_y).min(max_y);

                        let x = (left + right) * 0.5;
                        let y = (top + bottom) * 0.5;
                        let w = right - left;
                        let h = bottom - top;

                        self.batches.insert(
                            white_tex_id,
                            Some(UiArgs {
                                coords: [x, y].into(),
                                dimensions: [w, h].into(),
                                tex_coord_bounds: [0., 0., 1., 1.].into(),
                                color: tint.unwrap_or([1., 1., 1., 1.]).into(),
                                color_bias: [0., 0., 0., 0.].into(),
                            }),
                        )
                    }
                }

                if !glyph_data.vertices.is_empty() {
                    self.batches
                        .insert(glyph_tex_id, glyph_data.vertices.iter().cloned());
                }
            }
        }

        self.textures.maintain(factory, resources);
        changed = changed || self.batches.changed();

        {
            #[cfg(feature = "profiler")]
            profile_scope!("write");

            self.vertex.write(
                factory,
                index,
                self.batches.count() as u64,
                Some(self.batches.data()),
            );

            let view_args = UiViewArgs {
                inverse_window_size: [
                    1.0 / screen_dimensions.width() as f32,
                    1.0 / screen_dimensions.height() as f32,
                ]
                .into(),
            };
            changed = self.env.write(factory, index, view_args.std140()) || changed;
        }

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _resources: &GraphAuxData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw");

        if self.batches.count() > 0 {
            let layout = &self.pipeline_layout;
            encoder.bind_graphics_pipeline(&self.pipeline);
            self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);
            self.vertex.bind(index, 0, 0, &mut encoder);
            for (&tex, range) in self.batches.iter() {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &GraphAuxData) {
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
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), pso::CreationError> {
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
                .with_vertex_desc(&[(UiArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(pso::Primitive::TriangleStrip))
                .with_shaders(simple_shader_set(&shader_vertex, Some(&shader_fragment)))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: Some(pso::BlendState::ALPHA),
                }]),
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

fn mul_blend(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

fn render_image<B: Backend>(
    factory: &Factory<B>,
    aux: &GraphAuxData,
    transform: &UiTransform,
    raw_image: &UiImage,
    tint: &Option<[f32; 4]>,
    white_tex_id: TextureId,
    textures: &mut TextureSub<B>,
    batches: &mut OrderedOneLevelBatch<TextureId, UiArgs>,
) -> bool {
    let color = match (raw_image, tint.as_ref()) {
        (UiImage::SolidColor(color), Some(t)) => mul_blend(color, t),
        (UiImage::SolidColor(color), None) => *color,
        (_, Some(t)) => *t,
        (_, None) => [1., 1., 1., 1.],
    };

    let tex_coords = match raw_image {
        UiImage::Sprite(sprite_renderer) => {
            let sprite_sheets = aux.resources.get::<AssetStorage<SpriteSheet>>().unwrap();
            if let Some(sprite_sheet) = sprite_sheets.get(&sprite_renderer.sprite_sheet) {
                let sprites_storage = aux.resources.get::<AssetStorage<Sprites>>().unwrap();
                let sprites = sprites_storage.get(&sprite_sheet.sprites).unwrap();
                let tex_coord = &sprites.build_sprites()[sprite_renderer.sprite_number].tex_coords;
                [
                    tex_coord.left,
                    tex_coord.top,
                    tex_coord.right,
                    tex_coord.bottom,
                ]
            } else {
                [0.0_f32, 0., 1., 1.]
            }
        }
        UiImage::PartialTexture {
            left,
            right,
            bottom,
            top,
            ..
        } => [*left, *top, *right, *bottom],
        _ => [0.0_f32, 0., 1., 1.],
    };

    let args = UiArgs {
        coords: [transform.pixel_x(), transform.pixel_y()].into(),
        dimensions: [transform.pixel_width, transform.pixel_height].into(),
        tex_coord_bounds: tex_coords.into(),
        color: color.into(),
        color_bias: [0., 0., 0., 0.].into(),
    };

    match raw_image {
        UiImage::Texture(tex) => {
            if let Some((tex_id, this_changed)) = textures.insert(
                factory,
                aux.resources,
                tex,
                hal::image::Layout::ShaderReadOnlyOptimal,
            ) {
                batches.insert(tex_id, Some(args));
                this_changed
            } else {
                false
            }
        }
        UiImage::PartialTexture { tex, .. } => {
            if let Some((tex_id, this_changed)) = textures.insert(
                factory,
                aux.resources,
                tex,
                hal::image::Layout::ShaderReadOnlyOptimal,
            ) {
                batches.insert(tex_id, Some(args));
                this_changed
            } else {
                false
            }
        }
        UiImage::Sprite(sprite_renderer) => {
            let sprite_sheets = aux.resources.get::<AssetStorage<SpriteSheet>>().unwrap();
            if let Some(sprite_sheet) = sprite_sheets.get(&sprite_renderer.sprite_sheet) {
                if let Some((tex_id, this_changed)) = textures.insert(
                    factory,
                    aux.resources,
                    &sprite_sheet.texture,
                    hal::image::Layout::ShaderReadOnlyOptimal,
                ) {
                    batches.insert(tex_id, Some(args));
                    this_changed
                } else {
                    false
                }
            } else {
                false
            }
        }
        UiImage::NineSlice {
            x_start,
            y_start,
            width,
            height,
            left_dist,
            right_dist,
            top_dist,
            bottom_dist,
            tex,
            texture_dimensions,
        } => {
            if let Some((tex_id, this_changed)) = textures.insert(
                factory,
                aux.resources,
                tex,
                hal::image::Layout::ShaderReadOnlyOptimal,
            ) {
                //The texture locations of each slice
                let x_tex_coord_bound = [
                    *x_start as f32 / texture_dimensions[0] as f32,
                    (*x_start + *left_dist) as f32 / texture_dimensions[0] as f32,
                    (*x_start + *width - *right_dist) as f32 / texture_dimensions[0] as f32,
                    (*x_start + *width) as f32 / texture_dimensions[0] as f32,
                ];
                let y_tex_coord_bound = [
                    *y_start as f32 / texture_dimensions[1] as f32,
                    (*y_start + *top_dist) as f32 / texture_dimensions[1] as f32,
                    (*y_start + *height - *bottom_dist) as f32 / texture_dimensions[1] as f32,
                    (*y_start + *height) as f32 / texture_dimensions[1] as f32,
                ];

                //The dimensions of each slice
                let x_dimensions = [
                    *left_dist as f32,
                    transform.pixel_width - (*right_dist + *left_dist) as f32,
                    *right_dist as f32,
                ];
                let y_dimensions = [
                    *top_dist as f32,
                    transform.pixel_height - (*top_dist + *bottom_dist) as f32,
                    *bottom_dist as f32,
                ];

                //The center location of each slice on the screen
                let x_coords = [
                    transform.pixel_x() - ((transform.pixel_width - *left_dist as f32) / 2.0),
                    transform.pixel_x() + (*left_dist as f32 - *right_dist as f32) / 2.0,
                    transform.pixel_x() + ((transform.pixel_width - *right_dist as f32) / 2.0),
                ];
                let y_coords = [
                    transform.pixel_y() + ((transform.pixel_height - *top_dist as f32) / 2.0),
                    transform.pixel_y() + (*bottom_dist as f32 - *top_dist as f32) / 2.0,
                    transform.pixel_y() - ((transform.pixel_height - *bottom_dist as f32) / 2.0),
                ];

                // loop through left to right, the top to bottom and batch each slice to render
                for x in 0..3 {
                    for y in 0..3 {
                        let mut temp_args = args;
                        temp_args.tex_coord_bounds = [
                            x_tex_coord_bound[x],
                            y_tex_coord_bound[y],
                            x_tex_coord_bound[x + 1],
                            y_tex_coord_bound[y + 1],
                        ]
                        .into();
                        temp_args.dimensions = [x_dimensions[x], y_dimensions[y]].into();
                        temp_args.coords = [x_coords[x], y_coords[y]].into();
                        batches.insert(tex_id, Some(temp_args));
                    }
                }

                this_changed
            } else {
                false
            }
        }
        _ => {
            batches.insert(white_tex_id, Some(args));
            false
        }
    }
}

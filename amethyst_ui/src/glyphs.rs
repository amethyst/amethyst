//! Module containing the system managing glyphbrush state for visible UI Text components.

use std::{collections::HashMap, marker::PhantomData, ops::Deref};

use amethyst_assets::{
    AssetHandle, AssetStorage, DefaultLoader, Handle, LoadHandle, Loader, ProcessingQueue,
    ProcessingState,
};
use amethyst_core::{ecs::*, Hidden, HiddenPropagate};
use amethyst_rendy::{
    rendy::{
        command::QueueId,
        factory::{Factory, ImageState},
        hal,
        texture::{pixel::R8Unorm, TextureBuilder},
    },
    resources::Tint,
    Backend, Texture,
};
use glyph_brush::{
    rusttype::Scale, BrushAction, BrushError, BuiltInLineBreaker, FontId, GlyphBrush,
    GlyphBrushBuilder, GlyphCruncher, Layout, LineBreak, LineBreaker, SectionText, VariedSection,
};
use log::debug;
use serde::Deserialize;
use type_uuid::TypeUuid;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    format::FontData, get_default_font, pass::UiArgs, text::CachedGlyph, FontAsset, LineMode,
    Selected, TextEditing, UiText, UiTransform,
};

#[derive(Debug)]
pub struct UiGlyphsResource {
    glyph_tex: Option<Handle<Texture>>,
    default_font: Handle<FontAsset>,
}

impl UiGlyphsResource {
    pub fn new(resources: &Resources) -> Self {
        let loader = resources
            .get::<DefaultLoader>()
            .expect("Could not get Loader resource");

        let font_storage = resources.get::<ProcessingQueue<FontData>>().unwrap();

        Self {
            glyph_tex: None,
            default_font: get_default_font(&loader, &font_storage),
        }
    }
}

impl UiGlyphsResource {
    pub fn glyph_tex(&self) -> Option<&Handle<Texture>> {
        self.glyph_tex.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct UiGlyphs {
    pub(crate) sel_vertices: Vec<UiArgs>,
    pub(crate) vertices: Vec<UiArgs>,
    // props below are only filled for selected fields
    pub(crate) cursor_pos: (f32, f32),
    pub(crate) height: f32,
    pub(crate) space_width: f32,
}

#[derive(Debug)]
enum FontState {
    NotFound,
    Ready(FontId),
}

impl FontState {
    fn id(&self) -> Option<FontId> {
        match self {
            FontState::NotFound => None,
            FontState::Ready(id) => Some(*id),
        }
    }
}

#[derive(Debug, Hash, Clone, Copy)]
enum CustomLineBreaker {
    BuiltIn(BuiltInLineBreaker),
    None,
}

impl LineBreaker for CustomLineBreaker {
    fn line_breaks<'a>(&self, glyph_info: &'a str) -> Box<dyn Iterator<Item = LineBreak> + 'a> {
        match self {
            CustomLineBreaker::BuiltIn(inner) => inner.line_breaks(glyph_info),
            CustomLineBreaker::None => Box::new(std::iter::empty()),
        }
    }
}

/// Manages the text editing cursor create, deletion and position.
#[derive(Debug)]
pub struct UiGlyphsSystem<B: Backend> {
    glyph_brush: GlyphBrush<'static, (u32, UiArgs)>,
    glyph_entity_cache: HashMap<u32, Entity>,
    fonts_map: HashMap<LoadHandle, FontState>,
    marker: PhantomData<B>,
}

impl<B: Backend> Default for UiGlyphsSystem<B> {
    fn default() -> Self {
        debug!("Initializing UiGlyphsSystem");
        Self {
            glyph_brush: GlyphBrushBuilder::using_fonts(vec![])
                .initial_cache_size((512, 512))
                .build(),
            marker: PhantomData,
            fonts_map: HashMap::new(),
            glyph_entity_cache: HashMap::new(),
        }
    }
}

use derivative::Derivative;

#[derive(Debug, Clone, TypeUuid, Deserialize)]
#[uuid = "36e442d3-b957-4155-8f3b-01f580931226"]
pub struct GlyphTextureData {
    w: u32,
    h: u32,
}

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub(crate) struct GlyphTextureProcessorSystem<B> {
    pub(crate) _marker: std::marker::PhantomData<B>,
}

impl<B: Backend> System for GlyphTextureProcessorSystem<B> {
    fn build(self) -> Box<dyn ParallelRunnable> {
        use hal::format::{Component as C, Swizzle};

        Box::new(
            SystemBuilder::new("TextureProcessorSystem")
                .write_resource::<ProcessingQueue<GlyphTextureData>>()
                .write_resource::<AssetStorage<Texture>>()
                .read_resource::<QueueId>()
                .write_resource::<Factory<B>>()
                .build(
                    move |_commands, _world, (processing_queue, tex_storage, queue, factory), _| {
                        processing_queue.process(tex_storage, |b, _, _| {
                            log::debug!("Creating new glyph texture with size ({}, {})", b.w, b.h);

                            TextureBuilder::new()
                                .with_kind(hal::image::Kind::D2(b.w, b.h, 1, 1))
                                .with_view_kind(hal::image::ViewKind::D2)
                                .with_data_width(b.w)
                                .with_data_height(b.h)
                                .with_data(vec![R8Unorm { repr: [0] }; (b.w * b.h) as _])
                                // This swizzle is required when working with `R8Unorm` on metal.
                                // Glyph texture is biased towards 1.0 using "color_bias" attribute instead.
                                .with_swizzle(Swizzle(C::Zero, C::Zero, C::Zero, C::R))
                                .build(
                                    ImageState {
                                        queue: **queue,
                                        stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                                        access: hal::image::Access::SHADER_READ,
                                        layout: hal::image::Layout::General,
                                    },
                                    factory,
                                )
                                .map(B::wrap_texture)
                                .map(ProcessingState::Loaded)
                                .map_err(|e| e.into())
                        });

                        tex_storage.process_custom_drop(|_| {});
                    },
                ),
        )
    }
}

impl<B: Backend> System for UiGlyphsSystem<B> {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("UiGlyphsSystem")
                .write_resource::<Factory<B>>()
                .read_resource::<QueueId>()
                .write_resource::<AssetStorage<Texture>>()
                .write_resource::<ProcessingQueue<GlyphTextureData>>()
                .read_resource::<AssetStorage<FontAsset>>()
                .write_resource::<UiGlyphsResource>()
                .read_resource::<DefaultLoader>()
                .with_query(
                    <(
                        Entity,
                        &UiTransform,
                        &mut UiText,
                        Option<&TextEditing>,
                        Option<&Tint>,
                    )>::query()
                    .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
                )
                .with_query(
                    <(
                        Entity,
                        &mut UiGlyphs,
                        &UiTransform,
                        &mut UiText,
                        &TextEditing,
                    )>::query()
                    .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
                )
                .with_query(<(Entity, &mut UiGlyphs)>::query())
                .with_query(<(Entity, &mut Selected)>::query())
                .build(
                    move |commands,
                          world,
                          (
                        factory,
                        fetch_queue,
                        tex_storage,
                        tex_queue,
                        font_storage,
                        glyphs_res,
                        loader,
                    ),
                          (
                        texts_not_hidden_query_with_optional_editing,
                        not_hidden_glyphs_query_with_editing,
                        glyphs_query,
                        selected_query,
                    )| {
                        let queue = **fetch_queue.deref();

                        let glyph_tex = {
                            glyphs_res.glyph_tex.get_or_insert_with(|| {
                                let (w, h) = self.glyph_brush.texture_dimensions();
                                loader.load_from_data(GlyphTextureData { w, h }, (), &tex_queue)
                            })
                        };

                        if let Some(mut tex) =
                            tex_storage.get(glyph_tex).and_then(B::unwrap_texture)
                        {
                            let (mut glyph_world, mut else_world) =
                                world.split_for_query(glyphs_query);
                            let (mut selected_world, mut else_world) =
                                else_world.split_for_query(selected_query);

                            texts_not_hidden_query_with_optional_editing.for_each_mut(
                                &mut else_world,
                                |(entity, transform, ui_text, editing, tint)| {
                                    ui_text.cached_glyphs.clear();
                                    let font_handle =
                                        ui_text.font.as_ref().unwrap_or(&glyphs_res.default_font);

                                    let font_asset =
                                        font_storage.get(font_handle).map(|font| font.0.clone());

                                    if let FontState::NotFound = self
                                        .fonts_map
                                        .entry(font_handle.load_handle())
                                        .or_insert(FontState::NotFound)
                                    {
                                        if let Some(font) = font_storage.get(font_handle) {
                                            log::debug!("Adding font to glyph brush.");
                                            let new_font = FontState::Ready(
                                                self.glyph_brush.add_font(font.0.clone()),
                                            );
                                            self.fonts_map
                                                .insert(font_handle.load_handle(), new_font);
                                        }
                                    }

                                    let font_lookup =
                                        self.fonts_map.get(&font_handle.load_handle()).unwrap();

                                    if let (Some(font_id), Some(font_asset)) =
                                        (font_lookup.id(), font_asset)
                                    {
                                        let tint_color = tint.map_or([1., 1., 1., 1.], |t| {
                                            let (r, g, b, a) = t.0.into_components();
                                            [r, g, b, a]
                                        });

                                        let base_color = mul_blend(&ui_text.color, &tint_color);

                                        let scale = Scale::uniform(ui_text.font_size);

                                        let text = match (ui_text.password, editing) {
                                            (false, None) => {
                                                vec![SectionText {
                                                    text: &ui_text.text,
                                                    scale,
                                                    color: base_color,
                                                    font_id,
                                                }]
                                            }
                                            (false, Some(sel)) => {
                                                if let Some((start, end)) =
                                                    selection_span(sel, &ui_text.text)
                                                {
                                                    vec![
                                                        SectionText {
                                                            text: &ui_text.text[..start],
                                                            scale,
                                                            color: base_color,
                                                            font_id,
                                                        },
                                                        SectionText {
                                                            text: &ui_text.text[start..end],
                                                            scale,
                                                            color: mul_blend(
                                                                &sel.selected_text_color,
                                                                &tint_color,
                                                            ),
                                                            font_id,
                                                        },
                                                        SectionText {
                                                            text: &ui_text.text[end..],
                                                            scale,
                                                            color: base_color,
                                                            font_id,
                                                        },
                                                    ]
                                                } else {
                                                    vec![SectionText {
                                                        text: &ui_text.text,
                                                        scale,
                                                        color: base_color,
                                                        font_id,
                                                    }]
                                                }
                                            }
                                            (true, None) => {
                                                let string_len =
                                                    ui_text.text.graphemes(true).count();
                                                password_sections(string_len)
                                                    .map(|text| {
                                                        SectionText {
                                                            text,
                                                            scale,
                                                            color: base_color,
                                                            font_id,
                                                        }
                                                    })
                                                    .collect()
                                            }
                                            (true, Some(sel)) => {
                                                let string_len =
                                                    ui_text.text.graphemes(true).count();
                                                let pos = sel.cursor_position;
                                                let pos_highlight =
                                                    sel.cursor_position + sel.highlight_vector;
                                                let start = pos.min(pos_highlight) as usize;
                                                let to_end =
                                                    pos.max(pos_highlight) as usize - start;
                                                let rest = string_len - start - to_end;
                                                [
                                                    (start, base_color),
                                                    (
                                                        to_end,
                                                        mul_blend(
                                                            &sel.selected_text_color,
                                                            &tint_color,
                                                        ),
                                                    ),
                                                    (rest, base_color),
                                                ]
                                                .iter()
                                                .cloned()
                                                .flat_map(|(subsection_len, color)| {
                                                    password_sections(subsection_len).map(
                                                        move |text| {
                                                            SectionText {
                                                                text,
                                                                scale,
                                                                color,
                                                                font_id,
                                                            }
                                                        },
                                                    )
                                                })
                                                .collect()
                                            }
                                        };

                                        let layout = match ui_text.line_mode {
                                            LineMode::Single => {
                                                Layout::SingleLine {
                                                    line_breaker: CustomLineBreaker::None,
                                                    h_align: ui_text.align.horizontal_align(),
                                                    v_align: ui_text.align.vertical_align(),
                                                }
                                            }
                                            LineMode::Wrap => {
                                                Layout::Wrap {
                                                    line_breaker: CustomLineBreaker::BuiltIn(
                                                        BuiltInLineBreaker::UnicodeLineBreaker,
                                                    ),
                                                    h_align: ui_text.align.horizontal_align(),
                                                    v_align: ui_text.align.vertical_align(),
                                                }
                                            }
                                        };

                                        let next_z = if let Some(val) =
                                            self.glyph_entity_cache.keys().last()
                                        {
                                            val + 1
                                        } else {
                                            0
                                        };

                                        self.glyph_entity_cache.insert(next_z, *entity);

                                        let section = VariedSection {
                                            // Needs a recenter because we are using [-0.5,0.5] for the mesh
                                            // instead of the expected [0,1]
                                            screen_position: (
                                                transform.pixel_x
                                                    + transform.pixel_width
                                                        * ui_text.align.norm_offset().0,
                                                // invert y because layout calculates it in reverse
                                                -(transform.pixel_y
                                                    + transform.pixel_height
                                                        * ui_text.align.norm_offset().1),
                                            ),
                                            bounds: (transform.pixel_width, transform.pixel_height),
                                            // There is no other way to inject some glyph metadata than using Z.
                                            // Fortunately depth is not required, so this slot is instead used to
                                            // distinguish computed glyphs indented to be used for various entities
                                            // FIXME: This will be a problem because entities are now u64 and not u32....
                                            z: next_z as f32,
                                            layout: Default::default(), // overriden on queue
                                            text,
                                        };

                                        // `GlyphBrush::glyphs_custom_layout` does not return glyphs for invisible
                                        // characters.
                                        //
                                        // <https://docs.rs/glyph_brush/0.6.2/glyph_brush/trait.GlyphCruncher.html
                                        //  #tymethod.glyphs_custom_layout>
                                        //
                                        // For support, see:
                                        //
                                        // <https://github.com/alexheretic/glyph-brush/issues/80>
                                        let mut nonempty_cached_glyphs = self
                                            .glyph_brush
                                            .glyphs_custom_layout(&section, &layout)
                                            .map(|g| {
                                                let pos = g.position();
                                                let advance_width =
                                                    g.unpositioned().h_metrics().advance_width;
                                                CachedGlyph {
                                                    x: pos.x,
                                                    y: -pos.y,
                                                    advance_width,
                                                }
                                            });

                                        let mut last_cached_glyph: Option<CachedGlyph> = None;
                                        let all_glyphs = ui_text.text.chars().filter_map(|c| {
                                            if c.is_whitespace() {
                                                let (x, y) = if let Some(last_cached_glyph) =
                                                    last_cached_glyph
                                                {
                                                    let x = last_cached_glyph.x
                                                        + last_cached_glyph.advance_width;
                                                    let y = last_cached_glyph.y;
                                                    (x, y)
                                                } else {
                                                    (0.0, 0.0)
                                                };

                                                let advance_width = font_asset
                                                    .glyph(c)
                                                    .scaled(scale)
                                                    .h_metrics()
                                                    .advance_width;

                                                let cached_glyph = CachedGlyph {
                                                    x,
                                                    y,
                                                    advance_width,
                                                };
                                                last_cached_glyph = Some(cached_glyph);
                                                last_cached_glyph
                                            } else {
                                                last_cached_glyph = nonempty_cached_glyphs.next();
                                                last_cached_glyph
                                            }
                                        });
                                        ui_text.cached_glyphs.extend(all_glyphs);

                                        self.glyph_brush.queue_custom_layout(section, &layout);
                                    }
                                },
                            );

                            loop {
                                let action = self.glyph_brush.process_queued(
                                    |rect, data| unsafe {
                                        log::trace!("Upload glyph image at {:?}", rect);
                                        factory
                                            .upload_image(
                                                tex.image().clone(),
                                                rect.width(),
                                                rect.height(),
                                                hal::image::SubresourceLayers {
                                                    aspects: hal::format::Aspects::COLOR,
                                                    level: 0,
                                                    layers: 0..1,
                                                },
                                                hal::image::Offset {
                                                    x: rect.min.x as _,
                                                    y: rect.min.y as _,
                                                    z: 0,
                                                },
                                                hal::image::Extent {
                                                    width: rect.width(),
                                                    height: rect.height(),
                                                    depth: 1,
                                                },
                                                data,
                                                ImageState {
                                                    queue,
                                                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                                                    access: hal::image::Access::SHADER_READ,
                                                    layout: hal::image::Layout::General,
                                                },
                                                ImageState {
                                                    queue,
                                                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                                                    access: hal::image::Access::SHADER_READ,
                                                    layout: hal::image::Layout::General,
                                                },
                                            )
                                            .unwrap();
                                    },
                                    move |glyph| {
                                        // The glyph's Z parameter smuggles a key to retrieve entity, so glyphs can be associated
                                        // for rendering as part of specific components.
                                        let mut uv = glyph.tex_coords;
                                        let bounds_max_x = glyph.bounds.max.x as f32;
                                        let bounds_max_y = glyph.bounds.max.y as f32;
                                        let bounds_min_x = glyph.bounds.min.x as f32;
                                        let bounds_min_y = glyph.bounds.min.y as f32;
                                        let mut coords_max_x = glyph.pixel_coords.max.x as f32;
                                        let mut coords_max_y = glyph.pixel_coords.max.y as f32;
                                        let mut coords_min_x = glyph.pixel_coords.min.x as f32;
                                        let mut coords_min_y = glyph.pixel_coords.min.y as f32;

                                        // Glyph out of bounds, trim the quad
                                        if coords_max_x > bounds_max_x {
                                            let old_width = coords_max_x - coords_min_x;
                                            coords_max_x = bounds_max_x;
                                            uv.max.x = uv.min.x
                                                + (uv.max.x - uv.min.x)
                                                    * (coords_max_x - coords_min_x)
                                                    / old_width;
                                        }
                                        if coords_min_x < bounds_min_x {
                                            let old_width = coords_max_x - coords_min_x;
                                            coords_min_x = bounds_min_x;
                                            uv.min.x = uv.max.x
                                                - (uv.max.x - uv.min.x)
                                                    * (coords_max_x - coords_min_x)
                                                    / old_width;
                                        }
                                        if coords_max_y > bounds_max_y {
                                            let old_height = coords_max_y - coords_min_y;
                                            coords_max_y = bounds_max_y;
                                            uv.max.y = uv.min.y
                                                + (uv.max.y - uv.min.y)
                                                    * (coords_max_y - coords_min_y)
                                                    / old_height;
                                        }
                                        if coords_min_y < bounds_min_y {
                                            let old_height = coords_max_y - coords_min_y;
                                            coords_min_y = bounds_min_y;
                                            uv.min.y = uv.max.y
                                                - (uv.max.y - uv.min.y)
                                                    * (coords_max_y - coords_min_y)
                                                    / old_height;
                                        }

                                        let coords = [
                                            (coords_max_x + coords_min_x) * 0.5,
                                            -(coords_max_y + coords_min_y) * 0.5,
                                        ];
                                        let dims = [
                                            (coords_max_x - coords_min_x),
                                            (coords_max_y - coords_min_y),
                                        ];
                                        let tex_coord_bounds =
                                            [uv.min.x, uv.min.y, uv.max.x, uv.max.y];
                                        log::trace!(
                                            "Push glyph for cached glyph entity {}",
                                            glyph.z
                                        );
                                        (
                                            glyph.z as u32,
                                            UiArgs {
                                                coords: coords.into(),
                                                dimensions: dims.into(),
                                                tex_coord_bounds: tex_coord_bounds.into(),
                                                color: glyph.color.into(),
                                                color_bias: [1., 1., 1., 0.].into(),
                                            },
                                        )
                                    },
                                );

                                match action {
                                    Ok(BrushAction::Draw(vertices)) => {
                                        log::trace!("Updating glyph data, len {}", vertices.len());
                                        // entity ids are guaranteed to be in the same order as queued
                                        let mut glyph_ctr = 0;

                                        // make sure to erase all glyphs, even if not queued this frame

                                        for (_, glyph_data) in
                                            glyphs_query.iter_mut(&mut glyph_world)
                                        {
                                            glyph_data.vertices.clear();
                                            glyph_data.sel_vertices.clear();
                                        }

                                        texts_not_hidden_query_with_optional_editing.for_each_mut(
                                            &mut else_world,
                                            |(entity, transform, ui_text, editing, tint)| {
                                                let len = vertices[glyph_ctr..]
                                                    .iter()
                                                    .take_while(|(id, _)| {
                                                        self.glyph_entity_cache.get(id).unwrap()
                                                            == entity
                                                    })
                                                    .count();
                                                let entity_verts = vertices
                                                    [glyph_ctr..glyph_ctr + len]
                                                    .iter()
                                                    .map(|v| v.1);
                                                glyph_ctr += len;

                                                if let Ok((_, glyph_data)) =
                                                    glyphs_query.get_mut(&mut glyph_world, *entity)
                                                {
                                                    glyph_data.vertices.extend(entity_verts);
                                                } else {
                                                    commands.add_component(
                                                        *entity,
                                                        UiGlyphs {
                                                            vertices: entity_verts.collect(),
                                                            sel_vertices: vec![],
                                                            cursor_pos: (0., 0.),
                                                            height: 0.,
                                                            space_width: 0.,
                                                        },
                                                    );
                                                }

                                                let font_handle = ui_text
                                                    .font
                                                    .as_ref()
                                                    .unwrap_or(&glyphs_res.default_font);

                                                if let Some(editing) = editing {
                                                    if let Some(font) =
                                                        font_storage.get(font_handle)
                                                    {
                                                        let scale =
                                                            Scale::uniform(ui_text.font_size);
                                                        let v_metrics = font.0.v_metrics(scale);
                                                        let height =
                                                            v_metrics.ascent - v_metrics.descent;
                                                        let offset = (v_metrics.ascent
                                                            + v_metrics.descent)
                                                            * 0.5;
                                                        let total_len = ui_text.cached_glyphs.len();
                                                        let pos = editing.cursor_position;
                                                        let pos_highlight = editing.cursor_position
                                                            + editing.highlight_vector;
                                                        let start = (pos.min(pos_highlight)
                                                            as usize)
                                                            .min(total_len);
                                                        let end = (pos.max(pos_highlight) as usize)
                                                            .min(total_len);

                                                        let tint_color =
                                                            tint.map_or([1., 1., 1., 1.], |t| {
                                                                let (r, g, b, a) =
                                                                    t.0.into_components();
                                                                [r, g, b, a]
                                                            });
                                                        let bg_color =
                                                            editing.selected_background_color;
                                                        let bg_color = if selected_query
                                                            .get_mut(&mut selected_world, *entity)
                                                            .is_ok()
                                                        {
                                                            bg_color
                                                        } else {
                                                            mul_blend(
                                                                &bg_color,
                                                                &[0.5, 0.5, 0.5, 0.5],
                                                            )
                                                        };
                                                        let bg_color =
                                                            mul_blend(&tint_color, &bg_color);

                                                        let iter = ui_text.cached_glyphs
                                                            [start..end]
                                                            .iter()
                                                            .map(|g| {
                                                                UiArgs {
                                                                    coords: [
                                                                        g.x + g.advance_width * 0.5,
                                                                        g.y + offset,
                                                                    ]
                                                                    .into(),
                                                                    dimensions: [
                                                                        g.advance_width,
                                                                        height,
                                                                    ]
                                                                    .into(),
                                                                    tex_coord_bounds: [
                                                                        0., 0., 1., 1.,
                                                                    ]
                                                                    .into(),
                                                                    color: bg_color.into(),
                                                                    color_bias: [1., 1., 1., 0.]
                                                                        .into(),
                                                                }
                                                            });

                                                        if let Ok(glyph_data) = glyphs_query
                                                            .get_mut(&mut glyph_world, *entity)
                                                        {
                                                            glyph_data.1.sel_vertices.extend(iter);
                                                            glyph_data.1.height = height;
                                                            glyph_data.1.space_width = font
                                                                .0
                                                                .glyph(' ')
                                                                .scaled(scale)
                                                                .h_metrics()
                                                                .advance_width;
                                                            update_cursor_position(
                                                                glyph_data.1,
                                                                ui_text,
                                                                transform,
                                                                pos as usize,
                                                                offset,
                                                            );
                                                        }
                                                    }
                                                }
                                            },
                                        );
                                        break;
                                    }
                                    Ok(BrushAction::ReDraw) => {
                                        not_hidden_glyphs_query_with_editing.for_each_mut(
                                            world,
                                            |(_, glyph_data, transform, ui_text, editing)| {
                                                let font_handle = ui_text
                                                    .font
                                                    .as_ref()
                                                    .unwrap_or(&glyphs_res.default_font);

                                                if let Some(font) = font_storage.get(font_handle) {
                                                    let scale = Scale::uniform(ui_text.font_size);
                                                    let v_metrics = font.0.v_metrics(scale);
                                                    let pos = editing.cursor_position;
                                                    let offset = (v_metrics.ascent
                                                        + v_metrics.descent)
                                                        * 0.5;
                                                    update_cursor_position(
                                                        glyph_data,
                                                        ui_text,
                                                        transform,
                                                        pos as usize,
                                                        offset,
                                                    );
                                                }
                                            },
                                        );
                                        break;
                                    }
                                    Err(BrushError::TextureTooSmall { suggested: (w, h) }) => {
                                        // Replace texture in asset storage. No handles have to be updated.
                                        let glyph_tex: Handle<Texture> = loader.load_from_data(
                                            GlyphTextureData { w, h },
                                            (),
                                            &tex_queue,
                                        );

                                        tex = tex_storage
                                            .get(&glyph_tex)
                                            .and_then(B::unwrap_texture)
                                            .unwrap();
                                        self.glyph_brush.resize_texture(w, h);
                                    }
                                }
                            }
                        }
                    },
                ),
        )
    }
}

fn update_cursor_position(
    glyph_data: &mut UiGlyphs,
    ui_text: &UiText,
    transform: &UiTransform,
    pos: usize,
    offset: f32,
) {
    glyph_data.cursor_pos = if let Some(glyph) = ui_text.cached_glyphs.get(pos) {
        (glyph.x, glyph.y + offset)
    } else if let Some(glyph) = ui_text.cached_glyphs.last() {
        (glyph.x + glyph.advance_width, glyph.y + offset)
    } else {
        (
            transform.pixel_x() + transform.pixel_width * ui_text.align.norm_offset().0,
            transform.pixel_y(),
        )
    };
}

fn selection_span(editing: &TextEditing, string: &str) -> Option<(usize, usize)> {
    if editing.highlight_vector == 0 {
        return None;
    }

    let pos = editing.cursor_position;
    let pos_highlight = editing.cursor_position + editing.highlight_vector;

    let start = pos.min(pos_highlight) as usize;
    let to_end = pos.max(pos_highlight) as usize - start - 1;

    let mut indices = string.grapheme_indices(true).map(|i| i.0);
    let start_byte = indices.nth(start).unwrap_or_else(|| string.len());
    let end_byte = indices.nth(to_end).unwrap_or_else(|| string.len());

    if start_byte == end_byte {
        None
    } else {
        Some((start_byte, end_byte))
    }
}

fn mul_blend(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

const PASSWORD_STR: &str = "••••••••••••••••";
const PASSWORD_STR_GRAPHEMES: usize = 16; // 3 bytes per grapheme
fn password_sections(len: usize) -> impl Iterator<Item = &'static str> {
    let full_chunks = len / PASSWORD_STR_GRAPHEMES;
    let last_len = len % PASSWORD_STR_GRAPHEMES;
    std::iter::repeat(PASSWORD_STR)
        .take(full_chunks)
        .chain(Some(&PASSWORD_STR[0..last_len * 3]))
}

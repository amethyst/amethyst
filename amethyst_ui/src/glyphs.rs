//! Module containing the system managing glyphbrush state for visible UI Text components.

use crate::{
    pass::UiArgs, text::CachedGlyph, FontAsset, LineMode, Selected, TextEditing, UiText,
    UiTransform,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{
        Component, DenseVecStorage, Entities, Join, Read, ReadStorage, Resources, System,
        SystemData, Write, WriteExpect, WriteStorage,
    },
    Hidden, HiddenPropagate,
};
use amethyst_rendy::{
    rendy::{
        command::QueueId,
        factory::{Factory, ImageState},
        hal,
        texture::{pixel::R8Srgb, TextureBuilder},
    },
    resources::Tint,
    Backend, Texture,
};
use glyph_brush::{
    rusttype::Scale, BrushAction, BrushError, BuiltInLineBreaker, FontId, GlyphBrush,
    GlyphBrushBuilder, GlyphCruncher, Layout, LineBreak, LineBreaker, SectionText, VariedSection,
};
use std::{collections::HashMap, marker::PhantomData};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct UiGlyphsResource {
    glyph_tex: Option<Handle<Texture>>,
}

impl UiGlyphsResource {
    pub fn glyph_tex(&self) -> Option<&Handle<Texture>> {
        self.glyph_tex.as_ref()
    }
}

pub struct UiGlyphs {
    pub(crate) sel_vertices: Vec<UiArgs>,
    pub(crate) vertices: Vec<UiArgs>,
    // props below are only filled for selected fields
    pub(crate) cursor_pos: (f32, f32),
    pub(crate) height: f32,
    pub(crate) space_width: f32,
}

impl Component for UiGlyphs {
    type Storage = DenseVecStorage<Self>;
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
pub struct UiGlyphsSystem<B: Backend> {
    glyph_brush: GlyphBrush<'static, (u32, UiArgs)>,
    fonts_map: HashMap<u32, FontState>,
    marker: PhantomData<B>,
}

impl<B: Backend> UiGlyphsSystem<B> {
    /// Create new UI glyphs system
    pub fn new() -> Self {
        Self {
            glyph_brush: GlyphBrushBuilder::using_fonts(vec![])
                .initial_cache_size((512, 512))
                .build(),
            fonts_map: Default::default(),
            marker: PhantomData,
        }
    }
}

impl<'a, B: Backend> System<'a> for UiGlyphsSystem<B> {
    type SystemData = (
        Option<Write<'a, Factory<B>>>,
        Option<Read<'a, QueueId>>,
        Entities<'a>,
        ReadStorage<'a, UiTransform>,
        WriteStorage<'a, UiText>,
        WriteStorage<'a, UiGlyphs>,
        ReadStorage<'a, TextEditing>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        ReadStorage<'a, Selected>,
        ReadStorage<'a, Tint>,
        Write<'a, AssetStorage<Texture>>,
        Read<'a, AssetStorage<FontAsset>>,
        WriteExpect<'a, UiGlyphsResource>,
    );

    fn run(
        &mut self,
        (
            mut maybe_factory,
            maybe_queue,
            entities,
            transforms,
            mut texts,
            mut glyphs,
            text_editings,
            hiddens,
            hidden_propagates,
            selecteds,
            tints,
            mut tex_storage,
            font_storage,
            mut glyphs_res,
        ): Self::SystemData,
    ) {
        let (factory, queue) =
            if let (Some(factory), Some(queue)) = (maybe_factory.as_mut(), maybe_queue) {
                (factory, queue)
            } else {
                // Rendering system not present which might be the case during testing.
                // Just do nothing.
                return;
            };

        let glyph_tex = glyphs_res.glyph_tex.get_or_insert_with(|| {
            let (w, h) = self.glyph_brush.texture_dimensions();
            tex_storage.insert(create_glyph_texture(factory, *queue, w, h))
        });

        let mut tex = tex_storage
            .get(glyph_tex)
            .and_then(B::unwrap_texture)
            .expect("Glyph texture is created synchronously");

        let fonts_map_ref = &mut self.fonts_map;
        let glyph_brush_ref = &mut self.glyph_brush;

        for (entity, transform, ui_text, editing, tint, _, _) in (
            &entities,
            &transforms,
            &mut texts,
            text_editings.maybe(),
            tints.maybe(),
            !&hiddens,
            !&hidden_propagates,
        )
            .join()
        {
            let font_lookup = fonts_map_ref.entry(ui_text.font.id()).or_insert_with(|| {
                if let Some(font) = font_storage.get(&ui_text.font) {
                    FontState::Ready(glyph_brush_ref.add_font(font.0.clone()))
                } else {
                    FontState::NotFound
                }
            });

            ui_text.cached_glyphs.clear();

            if let Some(font_id) = font_lookup.id() {
                let tint_color = tint.map_or([1., 1., 1., 1.], |t| {
                    let (r, g, b, a) = t.0.into_components();
                    [r, g, b, a]
                });
                let base_color = mul_blend(&ui_text.color, &tint_color);

                let scale = Scale::uniform(ui_text.font_size);

                let text = match (ui_text.password, editing) {
                    (false, None) => vec![SectionText {
                        text: &ui_text.text,
                        scale,
                        color: base_color,
                        font_id,
                    }],
                    (false, Some(sel)) => {
                        if let Some((start, end)) = selection_span(sel, &ui_text.text) {
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
                                    color: mul_blend(&sel.selected_text_color, &tint_color),
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
                        let string_len = ui_text.text.graphemes(true).count();
                        password_sections(string_len)
                            .map(|text| SectionText {
                                text,
                                scale,
                                color: base_color,
                                font_id,
                            })
                            .collect()
                    }
                    (true, Some(sel)) => {
                        let string_len = ui_text.text.graphemes(true).count();
                        let pos = sel.cursor_position;
                        let pos_highlight = sel.cursor_position + sel.highlight_vector;
                        let start = pos.min(pos_highlight) as usize;
                        let to_end = pos.max(pos_highlight) as usize - start;
                        let rest = string_len - start - to_end;
                        [
                            (start, base_color),
                            (to_end, mul_blend(&sel.selected_text_color, &tint_color)),
                            (rest, base_color),
                        ]
                        .iter()
                        .cloned()
                        .flat_map(|(subsection_len, color)| {
                            password_sections(subsection_len).map(move |text| SectionText {
                                text,
                                scale,
                                color,
                                font_id,
                            })
                        })
                        .collect()
                    }
                };

                let layout = match ui_text.line_mode {
                    LineMode::Single => Layout::SingleLine {
                        line_breaker: CustomLineBreaker::None,
                        h_align: ui_text.align.horizontal_align(),
                        v_align: ui_text.align.vertical_align(),
                    },
                    LineMode::Wrap => Layout::Wrap {
                        line_breaker: CustomLineBreaker::BuiltIn(
                            BuiltInLineBreaker::UnicodeLineBreaker,
                        ),
                        h_align: ui_text.align.horizontal_align(),
                        v_align: ui_text.align.vertical_align(),
                    },
                };

                let section = VariedSection {
                    // Needs a recenter because we are using [-0.5,0.5] for the mesh
                    // instead of the expected [0,1]
                    screen_position: (
                        transform.pixel_x + transform.pixel_width * ui_text.align.norm_offset().0,
                        // invert y because layout calculates it in reverse
                        -(transform.pixel_y
                            + transform.pixel_height * ui_text.align.norm_offset().1),
                    ),
                    bounds: (transform.pixel_width, transform.pixel_height),
                    // There is no other way to inject some glyph metadata than using Z.
                    // Fortunately depth is not required, so this slot is instead used to
                    // distinguish computed glyphs indented to be used for various entities.
                    z: unsafe { std::mem::transmute(entity.id()) },
                    layout: Default::default(), // overriden on queue
                    text,
                };

                ui_text.cached_glyphs.extend(
                    glyph_brush_ref
                        .glyphs_custom_layout(&section, &layout)
                        .map(|g| {
                            let pos = g.position();
                            let advance_width = g.unpositioned().h_metrics().advance_width;
                            CachedGlyph {
                                x: pos.x,
                                y: -pos.y,
                                advance_width,
                            }
                        }),
                );

                glyph_brush_ref.queue_custom_layout(section, &layout);
            }
        }

        loop {
            let action = glyph_brush_ref.process_queued(
                |rect, data| unsafe {
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
                                queue: *queue,
                                stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                                access: hal::image::Access::SHADER_READ,
                                layout: hal::image::Layout::General,
                            },
                            ImageState {
                                queue: *queue,
                                stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                                access: hal::image::Access::SHADER_READ,
                                layout: hal::image::Layout::General,
                            },
                        )
                        .unwrap();
                },
                move |glyph| {
                    // The glyph's Z parameter smuggles entity id, so glyphs can be associated
                    // for rendering as part of specific components.
                    let entity_id: u32 = unsafe { std::mem::transmute(glyph.z) };

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
                            + (uv.max.x - uv.min.x) * (coords_max_x - coords_min_x) / old_width;
                    }
                    if coords_min_x < bounds_min_x {
                        let old_width = coords_max_x - coords_min_x;
                        coords_min_x = bounds_min_x;
                        uv.min.x = uv.max.x
                            - (uv.max.x - uv.min.x) * (coords_max_x - coords_min_x) / old_width;
                    }
                    if coords_max_y > bounds_max_y {
                        let old_height = coords_max_y - coords_min_y;
                        coords_max_y = bounds_max_y;
                        uv.max.y = uv.min.y
                            + (uv.max.y - uv.min.y) * (coords_max_y - coords_min_y) / old_height;
                    }
                    if coords_min_y < bounds_min_y {
                        let old_height = coords_max_y - coords_min_y;
                        coords_min_y = bounds_min_y;
                        uv.min.y = uv.max.y
                            - (uv.max.y - uv.min.y) * (coords_max_y - coords_min_y) / old_height;
                    }

                    let coords = [
                        (coords_max_x + coords_min_x) * 0.5,
                        -(coords_max_y + coords_min_y) * 0.5,
                    ];
                    let dims = [(coords_max_x - coords_min_x), (coords_max_y - coords_min_y)];
                    let tex_coord_bounds = [uv.min.x, uv.min.y, uv.max.x, uv.max.y];
                    (
                        entity_id,
                        UiArgs {
                            coords: coords.into(),
                            dimensions: dims.into(),
                            tex_coord_bounds: tex_coord_bounds.into(),
                            color: glyph.color.into(),
                        },
                    )
                },
            );

            match action {
                Ok(BrushAction::Draw(vertices)) => {
                    // entity ids are guaranteed to be in the same order as queued
                    let mut glyph_ctr = 0;

                    // make sure to erase all glyphs, even if not queued this frame
                    for glyph_data in (&mut glyphs).join() {
                        glyph_data.vertices.clear();
                        glyph_data.sel_vertices.clear();
                    }

                    for (entity, ui_text, editing, tint, transform, _, _) in (
                        &entities,
                        &texts,
                        text_editings.maybe(),
                        tints.maybe(),
                        &transforms,
                        !&hiddens,
                        !&hidden_propagates,
                    )
                        .join()
                    {
                        let e_id = entity.id();
                        let len = vertices[glyph_ctr..]
                            .iter()
                            .take_while(|(id, _)| *id == e_id)
                            .count();
                        let entity_verts = vertices[glyph_ctr..glyph_ctr + len].iter().map(|v| v.1);
                        glyph_ctr += len;

                        if let Some(glyph_data) = glyphs.get_mut(entity) {
                            glyph_data.vertices.extend(entity_verts);
                        } else {
                            glyphs
                                .insert(
                                    entity,
                                    UiGlyphs {
                                        vertices: entity_verts.collect(),
                                        sel_vertices: vec![],
                                        cursor_pos: (0., 0.),
                                        height: 0.,
                                        space_width: 0.,
                                    },
                                )
                                .unwrap();
                        }

                        if let Some(editing) = editing {
                            let font = font_storage
                                .get(&ui_text.font)
                                .expect("Font with rendered glyphs must be loaded");
                            let scale = Scale::uniform(ui_text.font_size);
                            let v_metrics = font.0.v_metrics(scale);
                            let height = v_metrics.ascent - v_metrics.descent;
                            let offset = (v_metrics.ascent + v_metrics.descent) * 0.5;
                            let total_len = ui_text.cached_glyphs.len();
                            let pos = editing.cursor_position;
                            let pos_highlight = editing.cursor_position + editing.highlight_vector;
                            let start = (pos.min(pos_highlight) as usize).min(total_len);
                            let end = (pos.max(pos_highlight) as usize).min(total_len);

                            let tint_color = tint.map_or([1., 1., 1., 1.], |t| {
                                let (r, g, b, a) = t.0.into_components();
                                [r, g, b, a]
                            });
                            let bg_color = editing.selected_background_color;
                            let bg_color = if selecteds.contains(entity) {
                                bg_color
                            } else {
                                mul_blend(&bg_color, &[0.5, 0.5, 0.5, 0.5])
                            };
                            let bg_color = mul_blend(&tint_color, &bg_color);

                            let iter = ui_text.cached_glyphs[start..end].iter().map(|g| UiArgs {
                                coords: [g.x + g.advance_width * 0.5, g.y + offset].into(),
                                dimensions: [g.advance_width, height].into(),
                                tex_coord_bounds: [0., 0., 1., 1.].into(),
                                color: bg_color.clone().into(),
                            });
                            let mut glyph_data = glyphs.get_mut(entity).unwrap();
                            glyph_data.sel_vertices.extend(iter);
                            glyph_data.height = height;
                            glyph_data.space_width =
                                font.0.glyph(' ').scaled(scale).h_metrics().advance_width;
                            glyph_data.cursor_pos =
                                if let Some(glyph) = ui_text.cached_glyphs.get(pos as usize) {
                                    (glyph.x, glyph.y + offset)
                                } else if let Some(glyph) = ui_text.cached_glyphs.last() {
                                    (glyph.x + glyph.advance_width, glyph.y + offset)
                                } else {
                                    (
                                        transform.pixel_x()
                                            + transform.pixel_width * ui_text.align.norm_offset().0,
                                        transform.pixel_y(),
                                    )
                                };
                        }
                    }
                    break;
                }
                Ok(BrushAction::ReDraw) => {
                    break;
                }
                Err(BrushError::TextureTooSmall { suggested: (w, h) }) => {
                    // Replace texture in asset storage. No handles have to be updated.
                    tex_storage.replace(glyph_tex, create_glyph_texture(factory, *queue, w, h));
                    tex = tex_storage
                        .get(glyph_tex)
                        .and_then(B::unwrap_texture)
                        .unwrap();
                    glyph_brush_ref.resize_texture(w, h);
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        res.insert(UiGlyphsResource { glyph_tex: None });
    }
}

fn create_glyph_texture<B: Backend>(
    factory: &mut Factory<B>,
    queue: QueueId,
    w: u32,
    h: u32,
) -> Texture {
    use hal::format::{Component as C, Swizzle};
    TextureBuilder::new()
        .with_kind(hal::image::Kind::D2(w, h, 1, 1))
        .with_view_kind(hal::image::ViewKind::D2)
        .with_data_width(w)
        .with_data_height(h)
        .with_data(vec![R8Srgb { repr: [0] }; (w * h) as _])
        // TODO: This will not work properly on metal :(
        // need to add extra uniform and mask in shader for metal
        .with_swizzle(Swizzle(C::One, C::One, C::One, C::R))
        .build(
            ImageState {
                queue,
                stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                access: hal::image::Access::SHADER_READ,
                layout: hal::image::Layout::General,
            },
            factory,
        )
        .map(B::wrap_texture)
        .expect("Failed to create glyph texture")
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
    let start_byte = indices.nth(start).unwrap_or(string.len());
    let end_byte = indices.nth(to_end).unwrap_or(string.len());

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

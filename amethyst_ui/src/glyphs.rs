//! Module containing the system managing glyphbrush state for visible UI Text components.

use std::{collections::HashMap, marker::PhantomData};

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::*,
    Hidden, HiddenPropagate,
};
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
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    pass::UiArgs, text::CachedGlyph, FontAsset, LineMode, Selected, TextEditing, UiText,
    UiTransform,
};

#[derive(Debug)]
pub struct UiGlyphsResource {
    glyph_tex: Option<Handle<Texture>>,
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
#[allow(missing_debug_implementations)]
pub struct UiGlyphsSystemResource<B: Backend> {
    glyph_brush: GlyphBrush<'static, (u32, UiArgs)>,
    glyph_entity_cache: HashMap<u32, Entity>,
    fonts_map: HashMap<u32, FontState>,
    marker: PhantomData<B>,
}

impl<B: Backend> Default for UiGlyphsSystemResource<B> {
    fn default() -> Self {
        Self {
            glyph_brush: GlyphBrushBuilder::using_fonts(vec![])
                .initial_cache_size((512, 512))
                .build(),
            glyph_entity_cache: HashMap::new(),
            fonts_map: Default::default(),
            marker: PhantomData,
        }
    }
}

pub fn build_ui_glyphs_system<B: Backend> () -> impl Runnable {
        SystemBuilder::new("UiGlyphsSystem")
            .write_resource::<UiGlyphsSystemResource<B>>()
            .write_resource::<Option<Factory<B>>>()
            .read_resource::<Option<QueueId>>()
            .write_resource::<AssetStorage<Texture>>()
            .read_resource::<AssetStorage<FontAsset>>()
            .write_resource::<UiGlyphsResource>()
            .with_query(<(Entity, &UiTransform,&mut UiText, Option<&TextEditing>, Option<&Tint>)>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>()))
            .with_query(<(Entity, &mut UiGlyphs, &UiTransform,&mut UiText, &TextEditing)>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>()))
            .with_query(<(Entity, &mut UiGlyphs)>::query())
            .with_query(<(Entity, &mut Selected)>::query())
            .build(move |commands, world,
                         (resource, maybe_factory, maybe_queue, tex_storage, font_storage, glyphs_res),
                        (texts_not_hidden_query, not_hidden_glyphs_query_with_editing, glyphs_query, selected_query)| {

                // TODO : Post legion split this method because it's too huge ...

                let (mut factory, queue) =
                    if let (Some(factory), Some(queue)) = (maybe_factory.as_mut(), maybe_queue.as_ref()) {
                        (factory, queue)
                } else {
                    // Rendering system not present which might be the case during testing.
                    // Just do nothing.
                    log::trace!("Rendering not present: glyphs processing skipped");
                    return;
                };

                let glyph_tex = glyphs_res.glyph_tex.get_or_insert_with(|| {
                    let (w, h) = resource.glyph_brush.texture_dimensions();
                    tex_storage.insert(create_glyph_texture(&mut factory, *queue, w, h))
                });

                let mut tex = tex_storage
                    .get(glyph_tex)
                    .and_then(B::unwrap_texture)
                    .expect("Glyph texture is created synchronously");


                let (mut glyph_world, mut else_world) = world.split_for_query(glyphs_query);
                let (mut selected_world, mut else_world) = else_world.split_for_query(selected_query);

                texts_not_hidden_query.for_each_mut(&mut else_world, |(entity, transform, ui_text, editing, tint)| {
                    ui_text.cached_glyphs.clear();
                    let font_asset = font_storage.get(&ui_text.font).map(|font| font.0.clone());
                    if let FontState::NotFound = resource.fonts_map
                        .entry(ui_text.font.id())
                        .or_insert(FontState::NotFound) {
                        if let Some(font) = font_storage.get(&ui_text.font) {
                            let new_font = FontState::Ready(resource.glyph_brush.add_font(font.0.clone()));
                            resource.fonts_map.insert(ui_text.font.id(), new_font);
                        }
                    }

                    let font_lookup = resource.fonts_map
                        .get(&ui_text.font.id())
                        .unwrap();

                    if let (Some(font_id), Some(font_asset)) = (font_lookup.id(), font_asset) {
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

                        let next_z =
                            if let Some(val) =  resource.glyph_entity_cache.keys().last() {
                                val + 1
                            } else { 0 };

                        resource.glyph_entity_cache.insert(next_z, *entity);

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
                        let mut nonempty_cached_glyphs =  resource.glyph_brush.glyphs_custom_layout(&section, &layout)
                            .map(|g| {
                                let pos = g.position();
                                let advance_width = g.unpositioned().h_metrics().advance_width;
                                CachedGlyph {
                                    x: pos.x,
                                    y: -pos.y,
                                    advance_width,
                                }
                            });

                        let mut last_cached_glyph: Option<CachedGlyph> = None;
                        let all_glyphs = ui_text.text.chars().filter_map(|c| {
                            if c.is_whitespace() {
                                let (x, y) = if let Some(last_cached_glyph) = last_cached_glyph {
                                    let x = last_cached_glyph.x + last_cached_glyph.advance_width;
                                    let y = last_cached_glyph.y;
                                    (x, y)
                                } else {
                                    (0.0, 0.0)
                                };

                                let advance_width =
                                    font_asset.glyph(c).scaled(scale).h_metrics().advance_width;

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

                        resource.glyph_brush.queue_custom_layout(section, &layout);
                    }
                });

                loop {
                    let action = resource.glyph_brush.process_queued(
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
                            log::trace!("Push glyph for cached glyph entity {}", glyph.z);
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
                        }
                    );

                    match action {
                        Ok(BrushAction::Draw(vertices)) => {
                            log::trace!("Updating glyph data, len {}", vertices.len());
                            // entity ids are guaranteed to be in the same order as queued
                            let mut glyph_ctr = 0;

                            // make sure to erase all glyphs, even if not queued this frame

                            for (_, glyph_data) in glyphs_query.iter_mut(&mut glyph_world) {
                                glyph_data.vertices.clear();
                                glyph_data.sel_vertices.clear();
                            }

                            texts_not_hidden_query.for_each_mut(&mut else_world, |(entity, transform, ui_text, editing, tint)| {
                                let len = vertices[glyph_ctr..]
                                    .iter()
                                    .take_while(|(id, _)|  resource.glyph_entity_cache.get(id).unwrap() == entity)
                                    .count();
                                let entity_verts = vertices[glyph_ctr..glyph_ctr + len].iter().map(|v| v.1);
                                glyph_ctr += len;

                                if let Some((_, glyph_data)) = glyphs_query.get_mut(&mut glyph_world, *entity).ok() {
                                    glyph_data.vertices.extend(entity_verts);
                                } else {
                                    commands.add_component(*entity,
                                                           UiGlyphs {
                                                                vertices: entity_verts.collect(),
                                                                sel_vertices: vec![],
                                                                cursor_pos: (0., 0.),
                                                                height: 0.,
                                                                space_width: 0.,
                                                            });
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
                                    let bg_color = if selected_query.get_mut(&mut selected_world, *entity).is_ok() {
                                        bg_color
                                    } else {
                                        mul_blend(&bg_color, &[0.5, 0.5, 0.5, 0.5])
                                    };
                                    let bg_color = mul_blend(&tint_color, &bg_color);

                                    let iter = ui_text.cached_glyphs[start..end].iter().map(|g| UiArgs {
                                        coords: [g.x + g.advance_width * 0.5, g.y + offset].into(),
                                        dimensions: [g.advance_width, height].into(),
                                        tex_coord_bounds: [0., 0., 1., 1.].into(),
                                        color: bg_color.into(),
                                        color_bias: [1., 1., 1., 0.].into(),
                                    });
                                    let mut glyph_data = glyphs_query.get_mut(&mut glyph_world, *entity).unwrap();
                                    glyph_data.1.sel_vertices.extend(iter);
                                    glyph_data.1.height = height;
                                    glyph_data.1.space_width =
                                        font.0.glyph(' ').scaled(scale).h_metrics().advance_width;
                                    update_cursor_position(
                                        glyph_data.1,
                                        ui_text,
                                        transform,
                                        pos as usize,
                                        offset,
                                    );
                                }
                            });
                            break;
                        }
                        Ok(BrushAction::ReDraw) => {
                            not_hidden_glyphs_query_with_editing.for_each_mut(&mut else_world, |(_, glyph_data, transform, ui_text, editing)| {
                                let font = font_storage
                                    .get(&ui_text.font)
                                    .expect("Font with rendered glyphs must be loaded");
                                let scale = Scale::uniform(ui_text.font_size);
                                let v_metrics = font.0.v_metrics(scale);
                                let pos = editing.cursor_position;
                                let offset = (v_metrics.ascent + v_metrics.descent) * 0.5;
                                update_cursor_position(
                                    glyph_data,
                                    ui_text,
                                    transform,
                                    pos as usize,
                                    offset,
                                );
                            });
                            break;
                        }
                        Err(BrushError::TextureTooSmall { suggested: (w, h) }) => {
                            // Replace texture in asset storage. No handles have to be updated.
                            tex_storage.replace(glyph_tex, create_glyph_texture(&mut factory, *queue, w, h));
                            tex = tex_storage
                                .get(glyph_tex)
                                .and_then(B::unwrap_texture)
                                .unwrap();
                            resource.glyph_brush.resize_texture(w, h);
                        }
                    }
                }
            })
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

fn create_glyph_texture<B: Backend>(
    factory: &mut Factory<B>,
    queue: QueueId,
    w: u32,
    h: u32,
) -> Texture {
    use hal::format::{Component as C, Swizzle};
    log::trace!("Creating new glyph texture with size ({}, {})", w, h);

    TextureBuilder::new()
        .with_kind(hal::image::Kind::D2(w, h, 1, 1))
        .with_view_kind(hal::image::ViewKind::D2)
        .with_data_width(w)
        .with_data_height(h)
        .with_data(vec![R8Unorm { repr: [0] }; (w * h) as _])
        // This swizzle is required when working with `R8Unorm` on metal.
        // Glyph texture is biased towards 1.0 using "color_bias" attribute instead.
        .with_swizzle(Swizzle(C::Zero, C::Zero, C::Zero, C::R))
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

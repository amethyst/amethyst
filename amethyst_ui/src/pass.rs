//! Simple flat forward drawing pass.

use std::{
    cmp::{Ordering, PartialOrd},
    hash::{Hash, Hasher},
};

use derive_new::new;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use gfx::{preset::blend, pso::buffer::ElemStride, state::ColorMask};
use gfx_glyph::{
    BuiltInLineBreaker, FontId, GlyphBrush, GlyphBrushBuilder, GlyphCruncher, Layout, Point, Scale,
    SectionText, VariedSection,
};
use glsl_layout::{vec2, vec4, Uniform};
use hibitset::BitSet;
use log::error;
use unicode_segmentation::UnicodeSegmentation;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_assets::{AssetStorage, Handle, Loader};
use amethyst_core::specs::prelude::{
    Entities, Entity, Join, Read, ReadExpect, ReadStorage, WriteStorage,
};
use amethyst_renderer::{
    error::Result,
    pipe::{
        pass::{Pass, PassData},
        Effect, NewEffect,
    },
    Encoder, Factory, Hidden, HiddenPropagate, Mesh, PosTex, Resources, Rgba, ScreenDimensions,
    Shape, Texture, TextureData, TextureHandle, TextureMetadata, VertexFormat,
};

use super::*;

const VERT_SRC: &[u8] = include_bytes!("shaders/vertex.glsl");
const FRAG_SRC: &[u8] = include_bytes!("shaders/frag.glsl");

#[derive(Copy, Clone, Debug, Uniform)]
#[allow(dead_code)] // This is used by the shaders
#[repr(C)]
struct VertexArgs {
    invert_window_size: vec2,
    coord: vec2,
    dimension: vec2,
    color: vec4,
}

#[derive(Clone, Debug, Default)]
struct CachedDrawOrder {
    pub cached: BitSet,
    pub cache: Vec<(f32, Entity)>,
}

/// A color used to query a hashmap for a cached texture of that color.
struct KeyColor(pub [u8; 4]);

impl Eq for KeyColor {}

impl PartialEq for KeyColor {
    fn eq(&self, other: &Self) -> bool {
        self.0[0] == other.0[0]
            && self.0[1] == other.0[1]
            && self.0[2] == other.0[2]
            && self.0[3] == other.0[3]
    }
}

impl Hash for KeyColor {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Hash::hash_slice(&self.0, hasher);
    }
}

#[derive(new)]
/// Draw Ui elements.  UI won't display without this.  It's recommended this be your last pass.
pub struct DrawUi {
    #[new(default)]
    mesh: Option<Mesh>,
    #[new(default)]
    cached_draw_order: CachedDrawOrder,
    #[new(default)]
    cached_color_textures: HashMap<KeyColor, TextureHandle>,
    #[new(default)]
    glyph_brushes: GlyphBrushCache,
    #[new(default)]
    next_brush_cache_id: u64,
}

type GlyphBrushCache = HashMap<u64, GlyphBrush<'static, Resources, Factory>>;

impl<'a> PassData<'a> for DrawUi {
    type Data = (
        Entities<'a>,
        ReadExpect<'a, Loader>,
        ReadExpect<'a, ScreenDimensions>,
        Read<'a, AssetStorage<Texture>>,
        Read<'a, AssetStorage<FontAsset>>,
        ReadStorage<'a, Handle<Texture>>,
        ReadStorage<'a, UiTransform>,
        WriteStorage<'a, UiText>,
        ReadStorage<'a, TextEditing>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        ReadStorage<'a, Selected>,
        ReadStorage<'a, Rgba>,
    );
}

impl Pass for DrawUi {
    fn compile(&mut self, mut effect: NewEffect<'_>) -> Result<Effect> {
        #[cfg(feature = "profiler")]
        profile_scope!("ui_pass_build");

        // Initialize a single unit quad, we'll use this mesh when drawing quads later.
        // Centered around (0,0) and of size 2
        let data = Shape::Plane(None).generate_vertices::<Vec<PosTex>>(None);
        self.mesh = Some(Mesh::build(data).build(&mut effect.factory)?);

        // Create the effect
        use std::mem;
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer(
                "VertexArgs",
                mem::size_of::<<VertexArgs as Uniform>::Std140>(),
                1,
            )
            .with_raw_vertex_buffer(PosTex::ATTRIBUTES, PosTex::size() as ElemStride, 0)
            .with_texture("albedo")
            .with_blended_output("color", ColorMask::all(), blend::ALPHA, None)
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        factory: Factory,
        (
            entities,
            loader,
            screen_dimensions,
            tex_storage,
            font_storage,
            ui_image,
            ui_transform,
            mut ui_text,
            editing,
            hidden,
            hidden_prop,
            selecteds,
            rgba,
        ): <Self as PassData<'_>>::Data,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("ui_pass_apply");

        // Populate and update the draw order cache.
        {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_populatebitset");
            let bitset = &mut self.cached_draw_order.cached;
            self.cached_draw_order.cache.retain(|&(_z, entity)| {
                let keep = ui_transform.contains(entity);
                if !keep {
                    bitset.remove(entity.id());
                }
                keep
            });
        }

        for &mut (ref mut z, entity) in &mut self.cached_draw_order.cache {
            *z = ui_transform
                .get(entity)
                .expect("Unreachable: Enities are collected from a cache of prepopulate entities")
                .global_z;
        }

        // Attempt to insert the new entities in sorted position. Should reduce work during
        // the sorting step.
        let transform_set = ui_transform.mask().clone();
        {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_insertsorted");
            // Create a bitset containing only the new indices.
            let new = (&transform_set ^ &self.cached_draw_order.cached) & &transform_set;
            for (entity, transform, _new) in (&*entities, &ui_transform, &new).join() {
                let pos = self
                    .cached_draw_order
                    .cache
                    .iter()
                    .position(|&(cached_z, _)| transform.global_z >= cached_z);
                match pos {
                    Some(pos) => self
                        .cached_draw_order
                        .cache
                        .insert(pos, (transform.global_z, entity)),
                    None => self
                        .cached_draw_order
                        .cache
                        .push((transform.global_z, entity)),
                }
            }
        }
        self.cached_draw_order.cached = transform_set;

        // Sort from largest z value to smallest z value.
        // Most of the time this shouldn't do anything but you still need it for if the z values
        // change.
        {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_sortz");
            self.cached_draw_order
                .cache
                .sort_unstable_by(|&(z1, _), &(z2, _)| {
                    z1.partial_cmp(&z2).unwrap_or(Ordering::Equal)
                });
        }

        // Inverted screen dimensions. Used to scale from pixel coordinates to the opengl coordinates in the vertex shader.
        let invert_window_size = [
            1. / screen_dimensions.width(),
            1. / screen_dimensions.height(),
        ];

        let mesh = self
            .mesh
            .as_ref()
            .expect("`DrawUi::compile` was not called before `DrawUi::apply`");

        let vbuf = match mesh.buffer(PosTex::ATTRIBUTES) {
            Some(vbuf) => vbuf.clone(),
            None => return,
        };
        effect.data.vertex_bufs.push(vbuf);

        //Gather unused glyph brushes
        //These that are currently in use will be removed from this set.
        let mut unused_glyph_brushes = self
            .glyph_brushes
            .iter()
            .map(|(id, _)| *id)
            .collect::<HashSet<_>>();

        let highest_abs_z = {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_findhighestz");
            (&ui_transform,)
                .join()
                .map(|t| t.0.global_z)
                .fold(1.0, |highest, current| current.abs().max(highest))
        };
        for &(_z, entity) in &self.cached_draw_order.cache {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_draw_singleentity");
            // Do not render hidden entities.
            if hidden.contains(entity) || hidden_prop.contains(entity) {
                ui_text
                    .get_mut(entity)
                    .and_then(|ui_text| ui_text.brush_id)
                    .map(|brush_id| unused_glyph_brushes.remove(&brush_id));
                continue;
            }
            let ui_transform = ui_transform
                .get(entity)
                .expect("Unreachable: Entity is guaranteed to be present based on earlier actions");
            let rgba: [f32; 4] = rgba.get(entity).cloned().unwrap_or(Rgba::WHITE).into();
            if let Some(image) = ui_image
                .get(entity)
                .and_then(|image| tex_storage.get(&image))
            {
                #[cfg(feature = "profiler")]
                profile_scope!("ui_pass_draw_uiimage");
                let vertex_args = VertexArgs {
                    invert_window_size: invert_window_size.into(),
                    // Coordinates are middle centered. It makes it easier to do layouting in most cases.
                    coord: [ui_transform.pixel_x, ui_transform.pixel_y].into(),
                    dimension: [ui_transform.pixel_width, ui_transform.pixel_height].into(),
                    color: rgba.into(),
                };

                effect.update_constant_buffer("VertexArgs", &vertex_args.std140(), encoder);
                effect.data.textures.push(image.view().clone());
                effect.data.samplers.push(image.sampler().clone());
                effect.draw(mesh.slice(), encoder);
                effect.data.textures.clear();
                effect.data.samplers.clear();
            }

            if let Some(ui_text) = ui_text.get_mut(entity) {
                #[cfg(feature = "profiler")]
                profile_scope!("ui_pass_draw_uitext");
                // Maintain glyph brushes.
                if ui_text.brush_id.is_none() || ui_text.font != ui_text.cached_font {
                    let font = match font_storage.get(&ui_text.font) {
                        Some(font) => font,
                        None => continue,
                    };

                    self.glyph_brushes.insert(
                        self.next_brush_cache_id,
                        GlyphBrushBuilder::using_font(font.0.clone()).build(factory.clone()),
                    );

                    ui_text.brush_id = Some(self.next_brush_cache_id);
                    ui_text.cached_font = ui_text.font.clone();
                    self.next_brush_cache_id += 1;
                } else if let Some(brush_id) = ui_text.brush_id {
                    unused_glyph_brushes.remove(&brush_id);
                }

                // Build text sections.
                let editing = editing.get(entity);
                let password_string = if ui_text.password {
                    // Build a string composed of black dot characters.
                    let mut ret = String::with_capacity(ui_text.text.len());
                    for _grapheme in ui_text.text.graphemes(true) {
                        ret.push('\u{2022}');
                    }
                    Some(ret)
                } else {
                    None
                };
                let rendered_string = password_string.as_ref().unwrap_or(&ui_text.text);
                let hidpi = screen_dimensions.hidpi_factor() as f32;
                let size = ui_text.font_size;
                let scale = Scale::uniform(size);
                let text = editing
                    .and_then(|editing| {
                        if editing.highlight_vector == 0 {
                            return None;
                        }
                        let start = editing
                            .cursor_position
                            .min(editing.cursor_position + editing.highlight_vector)
                            as usize;
                        let end = editing
                            .cursor_position
                            .max(editing.cursor_position + editing.highlight_vector)
                            as usize;
                        let start_byte = rendered_string
                            .grapheme_indices(true)
                            .nth(start)
                            .map(|i| i.0);
                        let end_byte = rendered_string
                            .grapheme_indices(true)
                            .nth(end)
                            .map(|i| i.0)
                            .unwrap_or_else(|| rendered_string.len());
                        start_byte.map(|start_byte| (editing, (start_byte, end_byte)))
                    })
                    .map(|(editing, (start_byte, end_byte))| {
                        let base_color = multiply_colors(ui_text.color, rgba);
                        vec![
                            SectionText {
                                text: &((rendered_string)[0..start_byte]),
                                scale: scale,
                                color: base_color,
                                font_id: FontId(0),
                            },
                            SectionText {
                                text: &((rendered_string)[start_byte..end_byte]),
                                scale: scale,
                                color: multiply_colors(editing.selected_text_color, rgba),
                                font_id: FontId(0),
                            },
                            SectionText {
                                text: &((rendered_string)[end_byte..]),
                                scale: scale,
                                color: base_color,
                                font_id: FontId(0),
                            },
                        ]
                    })
                    .unwrap_or_else(|| {
                        vec![SectionText {
                            text: rendered_string,
                            scale: scale,
                            color: multiply_colors(ui_text.color, rgba),
                            font_id: FontId(0),
                        }]
                    });

                let layout = match ui_text.line_mode {
                    LineMode::Single => Layout::SingleLine {
                        line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                        h_align: ui_text.align.horizontal_align(),
                        v_align: ui_text.align.vertical_align(),
                    },
                    LineMode::Wrap => Layout::Wrap {
                        line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                        h_align: ui_text.align.horizontal_align(),
                        v_align: ui_text.align.vertical_align(),
                    },
                };

                let section = VariedSection {
                    // Needs a recenter because we are using [-0.5,0.5] for the mesh
                    // instead of the expected [0,1]
                    screen_position: (
                        (ui_transform.pixel_x
                            + ui_transform.pixel_width * ui_text.align.norm_offset().0),
                        // invert y because gfx-glyph inverts it back
                        (screen_dimensions.height()
                            - ui_transform.pixel_y
                            - ui_transform.pixel_height * ui_text.align.norm_offset().1),
                    ),
                    bounds: (ui_transform.pixel_width, ui_transform.pixel_height),
                    // Invert z because of gfx-glyph using z+ forward
                    z: ui_transform.global_z / highest_abs_z,
                    layout,
                    text,
                };

                // Render background highlight
                let brush = {
                    #[cfg(feature = "profiler")]
                    profile_scope!("ui_pass_draw_uitext_backgroundhighlight");
                    &mut self
                        .glyph_brushes
                        .get_mut(&ui_text.brush_id.expect("Unreachable: `ui_text.brush_id` is guarenteed to be set earlier in this function"))
                        .expect("Unable to get brush from `glyph_brushes`-map")
                };
                // Maintain the glyph cache (used by the input code).
                ui_text.cached_glyphs.clear();
                ui_text
                    .cached_glyphs
                    .extend(brush.glyphs(&section).cloned());
                let cache = &mut self.cached_color_textures;

                // Render text selection
                if let Some((texture, (start, end))) = editing.and_then(|ed| {
                    let start = ed
                        .cursor_position
                        .min(ed.cursor_position + ed.highlight_vector)
                        as usize;
                    let end = ed
                        .cursor_position
                        .max(ed.cursor_position + ed.highlight_vector)
                        as usize;
                    let color = multiply_colors(
                        if selecteds.contains(entity) {
                            ed.selected_background_color
                        } else {
                            multiply_colors(ed.selected_background_color, [0.5, 0.5, 0.5, 0.5])
                        },
                        rgba,
                    );
                    tex_storage
                        .get(&cached_color_texture(cache, color, &loader, &tex_storage))
                        .map(|tex| (tex, (start, end)))
                }) {
                    // Text selection rendering
                    #[cfg(feature = "profiler")]
                    profile_scope!("ui_pass_draw_uitext_rendertextselection");

                    effect.data.textures.push(texture.view().clone());
                    effect.data.samplers.push(texture.sampler().clone());
                    let ascent = brush
                        .fonts()
                        .get(0)
                        .expect("Unable to get first font of brush")
                        .v_metrics(Scale::uniform(ui_text.font_size))
                        .ascent;
                    for glyph in brush
                        .glyphs(&section)
                        .enumerate()
                        .filter(|&(i, _g)| start <= i && i < end)
                        .map(|(_i, g)| g)
                    {
                        let height = glyph.scale().y / hidpi;
                        let width = glyph.unpositioned().h_metrics().advance_width / hidpi;
                        let mut pos = glyph.position();
                        pos.x /= hidpi;
                        pos.y /= hidpi;
                        let vertex_args = VertexArgs {
                            invert_window_size: invert_window_size.into(),
                            // gfx-glyph uses y down so we need to convert to y up
                            coord: [
                                pos.x + width / 2.0,
                                screen_dimensions.height() - pos.y + ascent / 2.0,
                            ]
                            .into(),
                            dimension: [width, height].into(),
                            color: rgba.into(),
                        };
                        effect.update_constant_buffer("VertexArgs", &vertex_args.std140(), encoder);
                        effect.draw(mesh.slice(), encoder);
                    }
                    effect.data.textures.clear();
                    effect.data.samplers.clear();
                }
                // Render text
                {
                    #[cfg(feature = "profiler")]
                    profile_scope!("ui_pass_draw_uitext_rendertext");
                    brush.queue(section.clone());
                    if let Err(err) = brush.draw_queued(
                        encoder,
                        &effect.data.out_blends[0],
                        &effect
                            .data
                            .out_depth
                            .as_ref()
                            .expect("Unable to get depth of effect")
                            .0,
                    ) {
                        error!("Unable to draw text! Error: {:?}", err);
                    }
                }
                // Render cursor
                if selecteds.contains(entity) {
                    if let Some((texture, editing)) = editing.as_ref().and_then(|ed| {
                        tex_storage
                            .get(&cached_color_texture(
                                cache,
                                multiply_colors(ui_text.color, rgba),
                                &loader,
                                &tex_storage,
                            ))
                            .map(|tex| (tex, ed))
                    }) {
                        #[cfg(feature = "profiler")]
                        profile_scope!("ui_pass_draw_uitext_rendercursor");
                        let blink_on = editing.cursor_blink_timer < 0.25;
                        if editing.use_block_cursor || blink_on {
                            effect.data.textures.push(texture.view().clone());
                            effect.data.samplers.push(texture.sampler().clone());
                            // Calculate the width of a space for use with the block cursor.
                            let space_width = if editing.use_block_cursor {
                                brush
                                    .fonts()
                                    .get(0)
                                    .expect("Unable to get first font of brush")
                                    .glyph(' ')
                                    .scaled(Scale::uniform(ui_text.font_size))
                                    .h_metrics()
                                    .advance_width
                            } else {
                                // If we aren't using the block cursor, don't bother.
                                0.0
                            };
                            let ascent = brush
                                .fonts()
                                .get(0)
                                .expect("Unable to get first font of brush")
                                .v_metrics(Scale::uniform(ui_text.font_size))
                                .ascent;
                            let glyph_len = brush.glyphs(&section).count();
                            let (glyph, at_end) = if editing.cursor_position as usize >= glyph_len {
                                (brush.glyphs(&section).last(), true)
                            } else {
                                (
                                    brush.glyphs(&section).nth(editing.cursor_position as usize),
                                    false,
                                )
                            };
                            let (height, width) = if editing.use_block_cursor {
                                let height = if blink_on {
                                    ui_text.font_size
                                } else {
                                    ui_text.font_size / 10.0
                                };

                                (height, space_width)
                            } else {
                                (ui_text.font_size, 2.0)
                            };

                            let mut pos = glyph.map(|g| g.position()).unwrap_or(Point {
                                x: ui_transform.pixel_x
                                    + ui_transform.width * ui_text.align.norm_offset().0,
                                y: 0.0,
                            });
                            // gfx-glyph uses y down so we need to convert to y up
                            pos.y =
                                screen_dimensions.height() - ui_transform.pixel_y + ascent / 2.0;

                            let mut x = pos.x;
                            if let Some(glyph) = glyph {
                                if at_end {
                                    x += glyph.unpositioned().h_metrics().advance_width;
                                }
                            }
                            let mut y = pos.y;
                            if editing.use_block_cursor && !blink_on {
                                y -= ui_text.font_size * 0.9;
                            }
                            let vertex_args = VertexArgs {
                                invert_window_size: invert_window_size.into(),
                                coord: [x, screen_dimensions.height() - y + ascent / 2.0].into(),
                                dimension: [width, height].into(),
                                color: rgba.into(),
                            };
                            effect.update_constant_buffer(
                                "VertexArgs",
                                &vertex_args.std140(),
                                encoder,
                            );
                            effect.draw(mesh.slice(), encoder);
                        }
                        effect.data.textures.clear();
                        effect.data.samplers.clear();
                    }
                }
            }
        }

        for id in unused_glyph_brushes.drain() {
            self.glyph_brushes.remove(&id);
        }
    }
}

fn multiply_colors(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

fn cached_color_texture(
    cache: &mut HashMap<KeyColor, TextureHandle>,
    color: [f32; 4],
    loader: &Loader,
    storage: &AssetStorage<Texture>,
) -> TextureHandle {
    fn to_u8(input: f32) -> u8 {
        (input * 255.0).min(255.0) as u8
    }
    let key = KeyColor([
        to_u8(color[0]),
        to_u8(color[1]),
        to_u8(color[2]),
        to_u8(color[3]),
    ]);
    cache
        .entry(key)
        .or_insert_with(|| {
            let meta = TextureMetadata::srgb();
            let texture_data = TextureData::Rgba(color, meta);
            loader.load_from_data(texture_data, (), storage)
        })
        .clone()
}

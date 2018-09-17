//! Simple flat forward drawing pass.

use super::*;
use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::cgmath::vec4 as cg_vec4;
use amethyst_core::specs::prelude::{
    Entities, Entity, Join, Read, ReadExpect, ReadStorage, WriteStorage,
};
use amethyst_renderer::error::Result;
use amethyst_renderer::pipe::pass::{Pass, PassData};
use amethyst_renderer::pipe::{Effect, NewEffect};
use amethyst_renderer::{
    Encoder, Factory, Mesh, PosTex, Resources, ScreenDimensions, Texture, TextureData,
    TextureHandle, TextureMetadata, VertexFormat,
};
use fnv::FnvHashMap as HashMap;
use gfx::preset::blend;
use gfx::pso::buffer::ElemStride;
use gfx::state::ColorMask;
use gfx_glyph::{
    BuiltInLineBreaker, FontId, GlyphBrush, GlyphBrushBuilder, GlyphCruncher, HorizontalAlign,
    Layout, Point, Scale, SectionText, VariedSection, VerticalAlign,
};
use glsl_layout::{vec2, vec4, Uniform};
use hibitset::BitSet;
use std::cmp::{Ordering, PartialOrd};
use std::hash::{Hash, Hasher};
use unicode_segmentation::UnicodeSegmentation;

const VERT_SRC: &[u8] = include_bytes!("shaders/vertex.glsl");
const FRAG_SRC: &[u8] = include_bytes!("shaders/frag.glsl");

#[derive(Copy, Clone, Debug, Uniform)]
#[allow(dead_code)] // This is used by the shaders
#[repr(C)]
struct VertexArgs {
    proj_vec: vec4,
    coord: vec2,
    dimension: vec2,
}

#[derive(Clone, Debug)]
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

/// Draw Ui elements.  UI won't display without this.  It's recommended this be your last pass.
pub struct DrawUi {
    mesh: Option<Mesh>,
    cached_draw_order: CachedDrawOrder,
    cached_color_textures: HashMap<KeyColor, TextureHandle>,
    glyph_brushes: GlyphBrushCache,
    next_brush_cache_id: u64,
}

type GlyphBrushCache = HashMap<u64, GlyphBrush<'static, Resources, Factory>>;

impl DrawUi {
    /// Create instance of `DrawUi` pass
    pub fn new() -> Self {
        DrawUi {
            mesh: None,
            cached_draw_order: CachedDrawOrder {
                cached: BitSet::new(),
                cache: Vec::new(),
            },
            cached_color_textures: HashMap::default(),
            glyph_brushes: HashMap::default(),
            next_brush_cache_id: 0,
        }
    }
}

impl<'a> PassData<'a> for DrawUi {
    type Data = (
        Entities<'a>,
        ReadExpect<'a, Loader>,
        ReadExpect<'a, ScreenDimensions>,
        Read<'a, AssetStorage<Texture>>,
        Read<'a, AssetStorage<FontAsset>>,
        Read<'a, UiFocused>,
        ReadStorage<'a, UiImage>,
        ReadStorage<'a, UiTransform>,
        WriteStorage<'a, UiText>,
        ReadStorage<'a, TextEditing>,
    );
}

impl Pass for DrawUi {
    fn compile(&mut self, mut effect: NewEffect) -> Result<Effect> {
        // Initialize a single unit quad, we'll use this mesh when drawing quads later
        let data = vec![
            PosTex {
                position: [0., 1., 0.],
                tex_coord: [0., 0.],
            },
            PosTex {
                position: [1., 1., 0.],
                tex_coord: [1., 0.],
            },
            PosTex {
                position: [1., 0., 0.],
                tex_coord: [1., 1.],
            },
            PosTex {
                position: [0., 1., 0.],
                tex_coord: [0., 0.],
            },
            PosTex {
                position: [1., 0., 0.],
                tex_coord: [1., 1.],
            },
            PosTex {
                position: [0., 0., 0.],
                tex_coord: [0., 1.],
            },
        ];
        self.mesh = Some(Mesh::build(data).build(&mut effect.factory)?);
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
            focused,
            ui_image,
            ui_transform,
            mut ui_text,
            editing,
        ): <Self as PassData>::Data,
    ) {
        // Populate and update the draw order cache.
        {
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
            *z = ui_transform.get(entity).unwrap().global_z;
        }

        // Attempt to insert the new entities in sorted position.  Should reduce work during
        // the sorting step.
        let transform_set = ui_transform.mask().clone();
        {
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
        self.cached_draw_order
            .cache
            .sort_unstable_by(|&(z1, _), &(z2, _)| z2.partial_cmp(&z1).unwrap_or(Ordering::Equal));

        let proj_vec = cg_vec4(
            2. / screen_dimensions.width(),
            -2. / screen_dimensions.height(),
            -2.,
            1.,
        );

        let mesh = self.mesh.as_ref().unwrap();

        let vbuf = match mesh.buffer(PosTex::ATTRIBUTES) {
            Some(vbuf) => vbuf.clone(),
            None => return,
        };
        effect.data.vertex_bufs.push(vbuf);

        let highest_abs_z = (&ui_transform,)
            .join()
            .map(|t| t.0.global_z)
            .fold(1.0, |highest, current| current.abs().max(highest));
        for &(_z, entity) in &self.cached_draw_order.cache {
            // This won't panic as we guaranteed earlier these entities are present.
            let ui_transform = ui_transform.get(entity).unwrap();
            if let Some(image) = ui_image
                .get(entity)
                .and_then(|image| tex_storage.get(&image.texture))
            {
                let vertex_args = VertexArgs {
                    proj_vec: proj_vec.into(),
                    // Coordinates are middle centered. It makes it easier to do layouting in most cases.
                    coord: [
                        ui_transform.pixel_x - ui_transform.pixel_width / 2.0
                            + screen_dimensions.width() / 2.,
                        ui_transform.pixel_y - ui_transform.pixel_height / 2.0
                            + screen_dimensions.height() / 2.,
                    ].into(),
                    dimension: [ui_transform.pixel_width, ui_transform.pixel_height].into(),
                };
                effect.update_constant_buffer("VertexArgs", &vertex_args.std140(), encoder);
                effect.data.textures.push(image.view().clone());
                effect.data.samplers.push(image.sampler().clone());
                effect.draw(mesh.slice(), encoder);
                effect.data.textures.clear();
                effect.data.samplers.clear();
            }

            if let Some(ui_text) = ui_text.get_mut(entity) {
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
                            .unwrap_or(rendered_string.len());
                        start_byte.map(|start_byte| (editing, (start_byte, end_byte)))
                    })
                    .map(|(editing, (start_byte, end_byte))| {
                        vec![
                            SectionText {
                                text: &((rendered_string)[0..start_byte]),
                                scale: Scale::uniform(ui_text.font_size),
                                color: ui_text.color,
                                font_id: FontId(0),
                            },
                            SectionText {
                                text: &((rendered_string)[start_byte..end_byte]),
                                scale: Scale::uniform(ui_text.font_size),
                                color: editing.selected_text_color,
                                font_id: FontId(0),
                            },
                            SectionText {
                                text: &((rendered_string)[end_byte..]),
                                scale: Scale::uniform(ui_text.font_size),
                                color: ui_text.color,
                                font_id: FontId(0),
                            },
                        ]
                    })
                    .unwrap_or(vec![SectionText {
                        text: rendered_string,
                        scale: Scale::uniform(ui_text.font_size),
                        color: ui_text.color,
                        font_id: FontId(0),
                    }]);
                // TODO: If you're adding multi-line support you need to change this to use
                // Layout::Wrap.
                let layout = Layout::SingleLine {
                    line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                    h_align: HorizontalAlign::Left,
                    v_align: VerticalAlign::Top,
                };
                let section = VariedSection {
                    screen_position: (
                        ui_transform.pixel_x - ui_transform.pixel_width / 2.0
                            + screen_dimensions.width() / 2.,
                        ui_transform.pixel_y - ui_transform.pixel_height / 2.0
                            + screen_dimensions.height() / 2.,
                    ),
                    bounds: (ui_transform.pixel_width, ui_transform.pixel_height),
                    z: ui_transform.global_z / highest_abs_z,
                    layout,
                    text,
                };

                // Render background highlight
                let brush = &mut self
                    .glyph_brushes
                    .get_mut(&ui_text.brush_id.unwrap())
                    .unwrap();
                // Maintain the glyph cache (used by the input code).
                ui_text.cached_glyphs.clear();
                ui_text
                    .cached_glyphs
                    .extend(brush.glyphs(&section).cloned());
                let cache = &mut self.cached_color_textures;
                if let Some((texture, (start, end))) = editing.and_then(|ed| {
                    let start = ed
                        .cursor_position
                        .min(ed.cursor_position + ed.highlight_vector)
                        as usize;
                    let end = ed
                        .cursor_position
                        .max(ed.cursor_position + ed.highlight_vector)
                        as usize;
                    let color = if focused.entity == Some(entity) {
                        ed.selected_background_color
                    } else {
                        [
                            ed.selected_background_color[0] * 0.5,
                            ed.selected_background_color[1] * 0.5,
                            ed.selected_background_color[2] * 0.5,
                            ed.selected_background_color[3] * 0.5,
                        ]
                    };
                    tex_storage
                        .get(&cached_color_texture(cache, color, &loader, &tex_storage))
                        .map(|tex| (tex, (start, end)))
                }) {
                    effect.data.textures.push(texture.view().clone());
                    effect.data.samplers.push(texture.sampler().clone());
                    let ascent = brush
                        .fonts()
                        .get(0)
                        .unwrap()
                        .v_metrics(Scale::uniform(ui_text.font_size))
                        .ascent;
                    for glyph in brush
                        .glyphs(&section)
                        .enumerate()
                        .filter(|&(i, _g)| start <= i && i < end)
                        .map(|(_i, g)| g)
                    {
                        let height = glyph.scale().y;
                        let width = glyph.unpositioned().h_metrics().advance_width;
                        let pos = glyph.position();
                        let vertex_args = VertexArgs {
                            proj_vec: proj_vec.into(),
                            coord: [pos.x, pos.y - ascent].into(),
                            dimension: [width, height].into(),
                        };
                        effect.update_constant_buffer("VertexArgs", &vertex_args.std140(), encoder);
                        effect.draw(mesh.slice(), encoder);
                    }
                    effect.data.textures.clear();
                    effect.data.samplers.clear();
                }
                // Render text
                brush.queue(section.clone());
                if let Err(err) = brush.draw_queued(
                    encoder,
                    &effect.data.out_blends[0],
                    &effect.data.out_depth.as_ref().unwrap().0,
                ) {
                    error!("Unable to draw text! Error: {:?}", err);
                }
                // Render cursor
                if focused.entity == Some(entity) {
                    if let Some((texture, editing)) = editing.as_ref().and_then(|ed| {
                        tex_storage
                            .get(&cached_color_texture(
                                cache,
                                ui_text.color,
                                &loader,
                                &tex_storage,
                            ))
                            .map(|tex| (tex, ed))
                    }) {
                        let blink_on = editing.cursor_blink_timer < 0.5 / CURSOR_BLINK_RATE;
                        if editing.use_block_cursor || blink_on {
                            effect.data.textures.push(texture.view().clone());
                            effect.data.samplers.push(texture.sampler().clone());
                            // Calculate the width of a space for use with the block cursor.
                            let space_width = if editing.use_block_cursor {
                                brush
                                    .fonts()
                                    .get(0)
                                    .unwrap()
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
                                .unwrap()
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
                            let height;
                            let width;
                            if editing.use_block_cursor {
                                height = if blink_on {
                                    ui_text.font_size
                                } else {
                                    ui_text.font_size / 10.0
                                };
                                width = space_width;
                            } else {
                                height = ui_text.font_size;
                                width = 2.0;
                            }
                            let pos = glyph.map(|g| g.position()).unwrap_or(Point {
                                x: ui_transform.pixel_x - ui_transform.pixel_width / 2.0
                                    + screen_dimensions.width() / 2.,
                                y: ui_transform.pixel_y - ui_transform.pixel_height / 2.0
                                    + ascent
                                    + screen_dimensions.height() / 2.,
                            });
                            let mut x = pos.x;
                            if let Some(glyph) = glyph {
                                if at_end {
                                    x += glyph.unpositioned().h_metrics().advance_width;
                                }
                            }
                            let mut y = pos.y - ascent;
                            if editing.use_block_cursor && !blink_on {
                                y += ui_text.font_size * 0.9;
                            }
                            let vertex_args = VertexArgs {
                                proj_vec: proj_vec.into(),
                                coord: [x, y].into(),
                                dimension: [width, height].into(),
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
    }
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
            let meta = TextureMetadata {
                sampler: None,
                mip_levels: Some(1),
                size: Some((1, 1)),
                dynamic: false,
                format: None,
                channel: None,
            };
            let texture_data = TextureData::Rgba(color, meta);
            loader.load_from_data(texture_data, (), storage)
        })
        .clone()
}

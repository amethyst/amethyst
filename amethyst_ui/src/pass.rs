//! Simple flat forward drawing pass.

use std::cmp::{Ordering, PartialOrd};

use amethyst_assets::{AssetStorage, Loader};
use amethyst_renderer::{Encoder, Factory, Mesh, MeshHandle, PosTex, Resources, ScreenDimensions, Texture,
                        VertexFormat};
use amethyst_renderer::error::Result;
use amethyst_renderer::pipe::{Effect, NewEffect};
use amethyst_renderer::pipe::pass::{Pass, PassData};
use cgmath::vec4;
use gfx::preset::blend;
use gfx::pso::buffer::ElemStride;
use gfx::state::ColorMask;
use gfx_glyph::{BuiltInLineBreaker, FontId, GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Scale, SectionText, Layout, VariedSection, VerticalAlign};
use hibitset::BitSet;
use specs::{Entities, Entity, Fetch, Join, ReadStorage, WriteStorage};
use unicode_segmentation::UnicodeSegmentation;

use super::*;

const VERT_SRC: &[u8] = include_bytes!("shaders/vertex.glsl");
const FRAG_SRC: &[u8] = include_bytes!("shaders/frag.glsl");

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)] // This is used by the shaders
#[repr(C)]
struct VertexArgs {
    proj_vec: [f32; 4],
    coord: [f32; 2],
    dimension: [f32; 2],
}

#[derive(Clone, Debug)]
struct CachedDrawOrder {
    pub cached: BitSet,
    pub cache: Vec<(f32, Entity)>,
}

/// Draw Ui elements.  UI won't display without this.  Ir's recommended this be your last pass.
pub struct DrawUi {
    mesh_handle: MeshHandle,
    cached_draw_order: CachedDrawOrder,
    glyph_brushes: Vec<GlyphBrush<'static, Resources, Factory>>,
}

impl DrawUi
where
    Self: Pass,
{
    /// Create instance of `DrawUi` pass
    pub fn new(loader: &Loader, mesh_storage: &AssetStorage<Mesh>) -> Self {
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
        ].into();
        let mesh_handle = loader.load_from_data(data, (), mesh_storage);
        DrawUi {
            mesh_handle,
            cached_draw_order: CachedDrawOrder {
                cached: BitSet::new(),
                cache: Vec::new(),
            },
            glyph_brushes: Vec::new(),
        }
    }
}

impl<'a> PassData<'a> for DrawUi {
    type Data = (
        Entities<'a>,
        Fetch<'a, ScreenDimensions>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, AssetStorage<FontAsset>>,
        ReadStorage<'a, UiImage>,
        ReadStorage<'a, UiTransform>,
        WriteStorage<'a, UiText>,
        ReadStorage<'a, TextEditing>,
    );
}

impl Pass for DrawUi {
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        use std::mem;
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
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
        (entities, screen_dimensions, mesh_storage, tex_storage, font_storage, ui_image, ui_transform, mut ui_text, editing): (
            Entities<'a>,
            Fetch<'a, ScreenDimensions>,
            Fetch<'a, AssetStorage<Mesh>>,
            Fetch<'a, AssetStorage<Texture>>,
            Fetch<'a, AssetStorage<FontAsset>>,
            ReadStorage<'a, UiImage>,
            ReadStorage<'a, UiTransform>,
            WriteStorage<'a, UiText>,
            ReadStorage<'a, TextEditing>,
        ),
){
        // Populate and update the draw order cache.
        {
            let bitset = &mut self.cached_draw_order.cached;
            self.cached_draw_order.cache.retain(|&(_z, entity)| {
                let keep = ui_transform.get(entity).is_some();
                if !keep {
                    bitset.remove(entity.id());
                }
                keep
            });
        }


        for &mut (ref mut z, entity) in &mut self.cached_draw_order.cache {
            *z = ui_transform.get(entity).unwrap().z;
        }

        // Attempt to insert the new entities in sorted position.  Should reduce work during
        // the sorting step.
        let transform_set = ui_transform.check();
        {
            // Create a bitset containing only the new indices.
            let new = (&transform_set ^ &self.cached_draw_order.cached) & &transform_set;
            for (entity, transform, _new) in (&*entities, &ui_transform, &new).join() {
                let pos = self.cached_draw_order
                    .cache
                    .iter()
                    .position(|&(cached_z, _)| transform.z >= cached_z);
                match pos {
                    Some(pos) => self.cached_draw_order
                        .cache
                        .insert(pos, (transform.z, entity)),
                    None => self.cached_draw_order.cache.push((transform.z, entity)),
                }
            }
        }
        self.cached_draw_order.cached = transform_set;

        // Sort from largest z value to smallest z value.
        // Most of the time this shouldn't do anything but you still need it for if the z values
        // change.
        self.cached_draw_order
            .cache
            .sort_unstable_by(|&(z1, _), &(z2, _)| {
                z2.partial_cmp(&z1).unwrap_or(Ordering::Equal)
            });

        //let cached_draw_order = &cached_draw_order;

        let proj_vec = vec4(
            2. / screen_dimensions.width(),
            -2. / screen_dimensions.height(),
            -2.,
            1.,
        );

        for &(_z, entity) in &self.cached_draw_order.cache {
            // This won't panic as we guaranteed earlier these entities are present.
            let ui_transform = ui_transform.get(entity).unwrap();
            let mesh = match mesh_storage.get(&self.mesh_handle) {
                Some(mesh) => mesh,
                None => return,
            };
            let vbuf = match mesh.buffer(PosTex::ATTRIBUTES) {
                Some(vbuf) => vbuf.clone(),
                None => continue,
            };
            let vertex_args = VertexArgs {
                proj_vec: proj_vec.into(),
                coord: [ui_transform.x, ui_transform.y],
                dimension: [ui_transform.width, ui_transform.height],
            };
            effect.update_constant_buffer("VertexArgs", &vertex_args, encoder);
            effect.data.vertex_bufs.push(vbuf);
            if let Some(image) = ui_image
                .get(entity)
                .and_then(|image| tex_storage.get(&image.texture))
            {
                effect.data.textures.push(image.view().clone());
                effect.data.samplers.push(image.sampler().clone());
                effect.draw(mesh.slice(), encoder);
                effect.clear();
            }

            if let Some(ui_text) = ui_text.get_mut(entity) {
                // Maintain glyph brushes.
                if ui_text.brush_id.is_none() || ui_text.font != ui_text.cached_font {
                    let font = match font_storage.get(&ui_text.font) {
                        Some(font) => font,
                        None => continue,
                    };
                    let mut new_id = None;

                    // This piece of code should be used once Font implements Eq.
                    // Xaeroxe is going to try and contribute this implementation.
                    /*self.glyph_brushes.iter().position(|brush| {
                        // GlyphBrush guarantees font 0 will be available.
                        brush.fonts().get(&FontId(0)).unwrap() == font
                    });*/
                    if new_id.is_none() {
                        new_id = Some(self.glyph_brushes.len());
                        self.glyph_brushes.push(GlyphBrushBuilder::using_font(font.0.clone()).build(factory.clone()));
                    }
                    ui_text.brush_id = new_id;
                    ui_text.cached_font = ui_text.font.clone();
                }
                // Build text sections.
                let editing = editing.get(entity);
                let text = editing.and_then(|editing| {
                    if editing.text_selected.start == editing.text_selected.end {
                        return None;
                    }
                    let start_byte = ui_text.text.grapheme_indices(true).nth(editing.text_selected.start).map(|i| i.0);
                    let end_byte = ui_text.text.grapheme_indices(true).nth(editing.text_selected.end).map(|i| i.0);
                    start_byte.into_iter().zip(end_byte.into_iter()).next().map(|indices| (editing, indices))
                }).map(|(editing, (start_byte, end_byte))| {
                    vec![
                        SectionText {
                            text: &((&ui_text.text)[0..start_byte]),
                            scale: Scale::uniform(ui_text.font_size),
                            color: ui_text.color,
                            font_id: FontId(0),
                        },
                        SectionText {
                            text: &((&ui_text.text)[start_byte..end_byte]),
                            scale: Scale::uniform(ui_text.font_size),
                            color: editing.selected_text_color,
                            font_id: FontId(0),
                        },
                        SectionText {
                            text: &((&ui_text.text)[end_byte..]),
                            scale: Scale::uniform(ui_text.font_size),
                            color: ui_text.color,
                            font_id: FontId(0),
                        },
                    ]
                }).unwrap_or(
                    vec![
                        SectionText {
                            text: &ui_text.text,
                            scale: Scale::uniform(ui_text.font_size),
                            color: ui_text.color,
                            font_id: FontId(0),
                        }
                    ]
                );
                let layout = Layout::Wrap {
                    line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                    h_align: HorizontalAlign::Left,
                    v_align: VerticalAlign::Top,
                };
                let section = VariedSection {
                    screen_position: (ui_transform.x, ui_transform.y),
                    bounds: (ui_transform.width, ui_transform.height),
                    z: ui_transform.z,
                    layout,
                    text,
                };
                // TODO: Render background highlight here
                let brush = &mut self.glyph_brushes[ui_text.brush_id.unwrap()];
                brush.queue(section);
                if let Err(err) = brush.draw_queued(encoder, &effect.data.out_blends[0], &effect.data.out_depth.as_ref().unwrap().0) {
                    eprintln!("Unable to draw text! Error: {:?}", err);
                }
                // TODO: Render cursor here
            }
        }
    }
}

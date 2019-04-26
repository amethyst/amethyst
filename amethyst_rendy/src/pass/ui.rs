use std::{
    cmp::{Ordering, PartialOrd},
    hash::{Hash, Hasher},
    collections::HashMap,
};

use crate::{
    batch::GroupIterator,
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    pass::util,
    pod::{SpriteArgs, ViewArgs, UiViewArgs, UiArgs},
};
use hibitset::BitSet;
use amethyst_assets::AssetStorage;
use amethyst_core::{
    prelude::{
        Entities, Entity, Join, Read, ReadExpect, ReadStorage, WriteStorage,
    },
    ecs::{Join, Read, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use derivative::Derivative;
use fnv::FnvHashMap;
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{
            Layout, PrepareResult, SetLayout, SimpleGraphicsPipeline, SimpleGraphicsPipelineDesc,
        },
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{
        adapter::PhysicalDevice,
        buffer::Usage as BufferUsage,
        device::Device,
        format::Format,
        pso::{
            self,
            BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc, Descriptor,
            DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType, ElemStride, Element,
            EntryPoint, GraphicsShaderSet, InstanceRate, ShaderStageFlags, Specialization,
        },
        Backend,
    },
    memory::Write,
    mesh::AsVertex,
    resource::{Buffer, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    shader::Shader,
};
use smallvec::SmallVec;
use std::collections::hash_map::Entry;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawUiDesc;

impl DrawUiDesc {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> SimpleGraphicsPipelineDesc<B, Resources> for DrawUiDesc {
    type Pipeline = DrawUi<B>;

    fn load_shader_set<'a>(
        &self,
        storage: &'a mut Vec<B::ShaderModule>,
        factory: &mut Factory<B>,
        _aux: &Resources,
    ) -> GraphicsShaderSet<'a, B> {
        storage.clear();

        log::trace!("Loading UI shader '{:#?}'", *super::UI_VERTEX);
        storage.push(unsafe { super::UI_VERTEX.module(factory).unwrap() });

        log::trace!("Loading UI shader '{:#?}'", *super::UI_FRAGMENT);
        storage.push(unsafe { super::UI_FRAGMENT.module(factory).unwrap() });

        GraphicsShaderSet {
            vertex: EntryPoint {
                entry: "main",
                module: &storage[0],
                specialization: Specialization::default(),
            },
            fragment: Some(EntryPoint {
                entry: "main",
                module: &storage[1],
                specialization: Specialization::default(),
            }),
            hull: None,
            domain: None,
            geometry: None,
        }
    }

    fn colors(&self) -> Vec<ColorBlendDesc> {
        // TODO(happens): transparency color
        vec![ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA)]
    }

    fn depth_stencil(&self) -> Option<DepthStencilDesc> {
        // TODO(happens): transparency stencil
        Some(DepthStencilDesc {
            depth: pso::DepthTest::On {
                fun: pso::Comparison::Less,
                write: true,
            },
            depth_bounds: false,
            stencil: pso::StencilTest::Off,
        })
    }

    fn vertices(&self) -> Vec<(Vec<Element<Format>>, ElemStride, InstanceRate)> {
        vec![UiArgs::VERTEX.gfx_vertex_input_desc(0)]
    }

    fn layout(&self) -> Layout {
        Layout {
            sets: vec![
                SetLayout {
                    bindings: vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: ShaderStageFlags::GRAPHICS,
                        immutable_samplers: false,
                    }],
                },
                SetLayout {
                    bindings: vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    }],
                },
                SetLayout {
                    bindings: vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    }],
                },
            ],
            push_constants: vec![],
        }
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _resource: &Resources,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        _set_layouts: &[RendyHandle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, failure::Error> {
        Ok(DrawUi {
            per_image: Vec::with_capacity(4),
            ..Default::default(),
        })
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawUi<B: Backend> {
    per_image: Vec<PerImage<B>>,
    cached_draw_order: CachedDrawOrder,
    cached_color_textures: HashMap<KeyColor, TextureHandle>,
}

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
struct PerImage<B: Backend> {
    projview_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    projview_set: Option<Escape<DescriptorSet<B>>>,

    tex_set: Vec<Escape<DescriptorSet<B>>>,
    tex_id_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
}

#[derive(Clone, Debug, Default)]
#[derivative(Default(bound = ""))]
struct CachedDrawOrder {
    pub cached: BitSet,
    pub cache: Vec<(f32, Entity)>,
}

// TODO(happens): Do we already have this somewhere else and can reuise it?
struct KeyColor(pub [u8; 4]);

impl Default for KeyColor {
    pub fn default() -> Self {
        KeyColor([0, 0, 0, 0])
    }
}

impl Eq for KeyColor {}

impl PartialEq for KeyColor {
    fn eq(&self, other: &Self) -> bool {
        self.0[0] == other.0[0]
            && self.0[1] == other.0[1]
            && self.0[2] == other.0[2]
            && self.0[3] == other.0[3]
    }
}

// TODO(happens): Is there a benefit to implementing this manually?
impl Hash for KeyColor {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Hash::hash_slice(&self.0, hasher);
    }
}

impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawUi<B> {
    type Desc = DrawUiDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        set_layouts: &[RendyHandle<DescriptorSetLayout<B>>],
        index: usize,
        resources: &Resources,
    ) -> PrepareResult {
        let (
            entities,
            loader,
            screen_dimensions,
            texture_storage,
            font_assets_storage,
            textures,
            transforms,
            mut texts,
            text_editings,
            hiddens,
            hidden_propagates,
            selected,
            rgba,
        ) = <(
            Entities<'_>,
            ReadExpect<'_, Loader>,
            ReadExpect<'_, ScreenDimensions>,
            Read<'_, AssetStorage<Texture>>,
            Read<'_, AssetStorage<FontAsset>>,
            ReadStorage<'_, Handle<Texture>>,
            ReadStorage<'_, UiTransform>,
            WriteStorage<'_, UiText>,
            ReadStorage<'_, TextEditing>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, Selected>,
            ReadStorage<'_, Rgba>,
        ) as SystemData>::fetch(resources);
        #[cfg(feature = "profiler")]
        profile_scope!("ui_pass_apply");

        // Populate and update the draw order cache.
        {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_populatebitset");
            let bitset = &mut self.cached_draw_order.cached;
            self.cached_draw_order.cache.retain(|&(_z, entity)| {
                let keep = transforms.contains(entity);
                if !keep {
                    bitset.remove(entity.id());
                }
                keep
            });
        }

        for &mut (ref mut z, entity) in &mut self.cached_draw_order.cache {
            *z = transforms
                .get(entity)
                .expect("Unreachable: Enities are collected from a cache of prepopulate entities")
                .global_z;
        }

        // Attempt to insert the new entities in sorted position. Should reduce work during
        // the sorting step.
        let transform_set = transforms.mask().clone();
        {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_insertsorted");
            // Create a bitset containing only the new indices.
            let new = (&transform_set ^ &self.cached_draw_order.cached) & &transform_set;
            for (entity, transform, _new) in (&*entities, &transforms, &new).join() {
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

        // let mesh = self
        //     .mesh
        //     .as_ref()
        //     .expect("`DrawUi::compile` was not called before `DrawUi::apply`");

        // let vbuf = match mesh.buffer(PosTex::ATTRIBUTES) {
        //     Some(vbuf) => vbuf.clone(),
        //     None => return,
        // };
        // effect.data.vertex_bufs.push(vbuf);

        // //Gather unused glyph brushes
        // //These that are currently in use will be removed from this set.
        // let mut unused_glyph_brushes = self
        //     .glyph_brushes
        //     .iter()
        //     .map(|(id, _)| *id)
        //     .collect::<HashSet<_>>();

        let highest_abs_z = {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_findhighestz");
            (&transforms,)
                .join()
                .map(|t| t.0.global_z)
                // TODO(happens): Use max_by here?
                .fold(1.0, |highest, current| current.abs().max(highest))
        };

        for &(_z, entity) in &self.cached_draw_order.cache {
            #[cfg(feature = "profiler")]
            profile_scope!("ui_pass_draw_singleentity");

            // Skip hidden entities
            if hiddens.contains(entity) || hidden_propagates.contains(entity) {
                continue;
            }

            let ui_transform = transforms
                .get(entity)
                .expect("Unreachable: Entity is guaranteed to be present based on earlier actions");

            let rgba: [f32; 4] = rgba.get(entity).cloned().unwrap_or(Rgba::WHITE).into();

            if let Some(tex) = textures
                .get(entity)
                .and_then(|tex| texture_storage.get(&tex))
            {
                #[cfg(feature = "profiler")]
                profile_scope!("ui_pass_draw_uiimage");

                // TODO(happens): Draw texture with params:
                // invert window size -> inverted window size
                // [ui_transform.pixel_x, ui_transform.pixel_y] -> coords
                // [ui_transform.pixel_width, ui_transform.pixel_height] -> dimensions
                // color -> color

                // Why were these uniforms before?!
            }

            // TODO(happens): Text drawing
            if let Some(text) = texts.get_mut(entity) {
                // #[cfg(feature = "profiler")]
                // profile_scope!("ui_pass_draw_uitext");
                // // Maintain glyph brushes.
                // if ui_text.brush_id.is_none() || ui_text.font != ui_text.cached_font {
                //     let font = match font_storage.get(&ui_text.font) {
                //         Some(font) => font,
                //         None => continue,
                //     };

                //     self.glyph_brushes.insert(
                //         self.next_brush_cache_id,
                //         GlyphBrushBuilder::using_font(font.0.clone()).build(factory.clone()),
                //     );

                //     ui_text.brush_id = Some(self.next_brush_cache_id);
                //     ui_text.cached_font = ui_text.font.clone();
                //     self.next_brush_cache_id += 1;
                // } else if let Some(brush_id) = ui_text.brush_id {
                //     unused_glyph_brushes.remove(&brush_id);
                // }

                // // Build text sections.
                // let editing = editing.get(entity);
                // let password_string = if ui_text.password {
                //     // Build a string composed of black dot characters.
                //     let mut ret = String::with_capacity(ui_text.text.len());
                //     for _grapheme in ui_text.text.graphemes(true) {
                //         ret.push('\u{2022}');
                //     }
                //     Some(ret)
                // } else {
                //     None
                // };
                // let rendered_string = password_string.as_ref().unwrap_or(&ui_text.text);
                // let hidpi = screen_dimensions.hidpi_factor() as f32;
                // let size = ui_text.font_size;
                // let scale = Scale::uniform(size);
                // let text = editing
                //     .and_then(|editing| {
                //         if editing.highlight_vector == 0 {
                //             return None;
                //         }
                //         let start = editing
                //             .cursor_position
                //             .min(editing.cursor_position + editing.highlight_vector)
                //             as usize;
                //         let end = editing
                //             .cursor_position
                //             .max(editing.cursor_position + editing.highlight_vector)
                //             as usize;
                //         let start_byte = rendered_string
                //             .grapheme_indices(true)
                //             .nth(start)
                //             .map(|i| i.0);
                //         let end_byte = rendered_string
                //             .grapheme_indices(true)
                //             .nth(end)
                //             .map(|i| i.0)
                //             .unwrap_or_else(|| rendered_string.len());
                //         start_byte.map(|start_byte| (editing, (start_byte, end_byte)))
                //     })
                //     .map(|(editing, (start_byte, end_byte))| {
                //         let base_color = multiply_colors(ui_text.color, rgba);
                //         vec![
                //             SectionText {
                //                 text: &((rendered_string)[0..start_byte]),
                //                 scale: scale,
                //                 color: base_color,
                //                 font_id: FontId(0),
                //             },
                //             SectionText {
                //                 text: &((rendered_string)[start_byte..end_byte]),
                //                 scale: scale,
                //                 color: multiply_colors(editing.selected_text_color, rgba),
                //                 font_id: FontId(0),
                //             },
                //             SectionText {
                //                 text: &((rendered_string)[end_byte..]),
                //                 scale: scale,
                //                 color: base_color,
                //                 font_id: FontId(0),
                //             },
                //         ]
                //     })
                //     .unwrap_or_else(|| {
                //         vec![SectionText {
                //             text: rendered_string,
                //             scale: scale,
                //             color: multiply_colors(ui_text.color, rgba),
                //             font_id: FontId(0),
                //         }]
                //     });

                // let layout = match ui_text.line_mode {
                //     LineMode::Single => Layout::SingleLine {
                //         line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                //         h_align: ui_text.align.horizontal_align(),
                //         v_align: ui_text.align.vertical_align(),
                //     },
                //     LineMode::Wrap => Layout::Wrap {
                //         line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                //         h_align: ui_text.align.horizontal_align(),
                //         v_align: ui_text.align.vertical_align(),
                //     },
                // };

                // let section = VariedSection {
                //     // Needs a recenter because we are using [-0.5,0.5] for the mesh
                //     // instead of the expected [0,1]
                //     screen_position: (
                //         (ui_transform.pixel_x
                //             + ui_transform.pixel_width * ui_text.align.norm_offset().0),
                //         // invert y because gfx-glyph inverts it back
                //         (screen_dimensions.height()
                //             - ui_transform.pixel_y
                //             - ui_transform.pixel_height * ui_text.align.norm_offset().1),
                //     ),
                //     bounds: (ui_transform.pixel_width, ui_transform.pixel_height),
                //     // Invert z because of gfx-glyph using z+ forward
                //     z: ui_transform.global_z / highest_abs_z,
                //     layout,
                //     text,
                // };

                // // Render background highlight
                // let brush = {
                //     #[cfg(feature = "profiler")]
                //     profile_scope!("ui_pass_draw_uitext_backgroundhighlight");
                //     &mut self
                //         .glyph_brushes
                //         .get_mut(&ui_text.brush_id.expect("Unreachable: `ui_text.brush_id` is guarenteed to be set earlier in this function"))
                //         .expect("Unable to get brush from `glyph_brushes`-map")
                // };
                // // Maintain the glyph cache (used by the input code).
                // ui_text.cached_glyphs.clear();
                // ui_text
                //     .cached_glyphs
                //     .extend(brush.glyphs(&section).cloned());
                // let cache = &mut self.cached_color_textures;

                // // Render text selection
                // if let Some((texture, (start, end))) = editing.and_then(|ed| {
                //     let start = ed
                //         .cursor_position
                //         .min(ed.cursor_position + ed.highlight_vector)
                //         as usize;
                //     let end = ed
                //         .cursor_position
                //         .max(ed.cursor_position + ed.highlight_vector)
                //         as usize;
                //     let color = multiply_colors(
                //         if selecteds.contains(entity) {
                //             ed.selected_background_color
                //         } else {
                //             multiply_colors(ed.selected_background_color, [0.5, 0.5, 0.5, 0.5])
                //         },
                //         rgba,
                //     );
                //     tex_storage
                //         .get(&cached_color_texture(cache, color, &loader, &tex_storage))
                //         .map(|tex| (tex, (start, end)))
                // }) {
                //     // Text selection rendering
                //     #[cfg(feature = "profiler")]
                //     profile_scope!("ui_pass_draw_uitext_rendertextselection");

                //     effect.data.textures.push(texture.view().clone());
                //     effect.data.samplers.push(texture.sampler().clone());
                //     let ascent = brush
                //         .fonts()
                //         .get(0)
                //         .expect("Unable to get first font of brush")
                //         .v_metrics(Scale::uniform(ui_text.font_size))
                //         .ascent;
                //     for glyph in brush
                //         .glyphs(&section)
                //         .enumerate()
                //         .filter(|&(i, _g)| start <= i && i < end)
                //         .map(|(_i, g)| g)
                //     {
                //         let height = glyph.scale().y / hidpi;
                //         let width = glyph.unpositioned().h_metrics().advance_width / hidpi;
                //         let mut pos = glyph.position();
                //         pos.x /= hidpi;
                //         pos.y /= hidpi;
                //         let vertex_args = VertexArgs {
                //             invert_window_size: invert_window_size.into(),
                //             // gfx-glyph uses y down so we need to convert to y up
                //             coord: [
                //                 pos.x + width / 2.0,
                //                 screen_dimensions.height() - pos.y + ascent / 2.0,
                //             ]
                //             .into(),
                //             dimension: [width, height].into(),
                //             color: rgba.into(),
                //         };
                //         effect.update_constant_buffer("VertexArgs", &vertex_args.std140(), encoder);
                //         effect.draw(mesh.slice(), encoder);
                //     }
                //     effect.data.textures.clear();
                //     effect.data.samplers.clear();
                // }
                // // Render text
                // {
                //     #[cfg(feature = "profiler")]
                //     profile_scope!("ui_pass_draw_uitext_rendertext");
                //     brush.queue(section.clone());
                //     if let Err(err) = brush.draw_queued(
                //         encoder,
                //         &effect.data.out_blends[0],
                //         &effect
                //             .data
                //             .out_depth
                //             .as_ref()
                //             .expect("Unable to get depth of effect")
                //             .0,
                //     ) {
                //         error!("Unable to draw text! Error: {:?}", err);
                //     }
                // }
                // // Render cursor
                // if selecteds.contains(entity) {
                //     if let Some((texture, editing)) = editing.as_ref().and_then(|ed| {
                //         tex_storage
                //             .get(&cached_color_texture(
                //                 cache,
                //                 multiply_colors(ui_text.color, rgba),
                //                 &loader,
                //                 &tex_storage,
                //             ))
                //             .map(|tex| (tex, ed))
                //     }) {
                //         #[cfg(feature = "profiler")]
                //         profile_scope!("ui_pass_draw_uitext_rendercursor");
                //         let blink_on = editing.cursor_blink_timer < 0.25;
                //         if editing.use_block_cursor || blink_on {
                //             effect.data.textures.push(texture.view().clone());
                //             effect.data.samplers.push(texture.sampler().clone());
                //             // Calculate the width of a space for use with the block cursor.
                //             let space_width = if editing.use_block_cursor {
                //                 brush
                //                     .fonts()
                //                     .get(0)
                //                     .expect("Unable to get first font of brush")
                //                     .glyph(' ')
                //                     .scaled(Scale::uniform(ui_text.font_size))
                //                     .h_metrics()
                //                     .advance_width
                //             } else {
                //                 // If we aren't using the block cursor, don't bother.
                //                 0.0
                //             };
                //             let ascent = brush
                //                 .fonts()
                //                 .get(0)
                //                 .expect("Unable to get first font of brush")
                //                 .v_metrics(Scale::uniform(ui_text.font_size))
                //                 .ascent;
                //             let glyph_len = brush.glyphs(&section).count();
                //             let (glyph, at_end) = if editing.cursor_position as usize >= glyph_len {
                //                 (brush.glyphs(&section).last(), true)
                //             } else {
                //                 (
                //                     brush.glyphs(&section).nth(editing.cursor_position as usize),
                //                     false,
                //                 )
                //             };
                //             let (height, width) = if editing.use_block_cursor {
                //                 let height = if blink_on {
                //                     ui_text.font_size
                //                 } else {
                //                     ui_text.font_size / 10.0
                //                 };

                //                 (height, space_width)
                //             } else {
                //                 (ui_text.font_size, 2.0)
                //             };

                //             let mut pos = glyph.map(|g| g.position()).unwrap_or(Point {
                //                 x: ui_transform.pixel_x
                //                     + ui_transform.width * ui_text.align.norm_offset().0,
                //                 y: 0.0,
                //             });
                //             // gfx-glyph uses y down so we need to convert to y up
                //             pos.y =
                //                 screen_dimensions.height() - ui_transform.pixel_y + ascent / 2.0;

                //             let mut x = pos.x;
                //             if let Some(glyph) = glyph {
                //                 if at_end {
                //                     x += glyph.unpositioned().h_metrics().advance_width;
                //                 }
                //             }
                //             let mut y = pos.y;
                //             if editing.use_block_cursor && !blink_on {
                //                 y -= ui_text.font_size * 0.9;
                //             }
                //             let vertex_args = VertexArgs {
                //                 invert_window_size: invert_window_size.into(),
                //                 coord: [x, screen_dimensions.height() - y + ascent / 2.0].into(),
                //                 dimension: [width, height].into(),
                //                 color: rgba.into(),
                //             };
                //             effect.update_constant_buffer(
                //                 "VertexArgs",
                //                 &vertex_args.std140(),
                //                 encoder,
                //             );
                //             effect.draw(mesh.slice(), encoder);
                //         }
                //         effect.data.textures.clear();
                //         effect.data.samplers.clear();
                //     }
                // }
            }
        }

        // for id in unused_glyph_brushes.drain() {
        //     self.glyph_brushes.remove(&id);
        // }

        PrepareResult::Reuse
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _resources: &Resources,
    ) {}

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {}
}

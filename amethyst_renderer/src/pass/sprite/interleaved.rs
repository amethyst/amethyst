//! Flat forward drawing pass that mimics a blit.

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::cgmath::Vector4;
use amethyst_core::specs::prelude::{Join, Read, ReadStorage};
use amethyst_core::transform::GlobalTransform;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;

use super::*;
use cam::{ActiveCamera, Camera};
use error::Result;
use mtl::MaterialTextureSet;
use pass::util::{
    add_texture, get_camera, set_sprite_args, set_view_args, setup_textures, SpriteArgs,
    TextureOffsetPod, ViewArgs,
};
use pipe::pass::{Pass, PassData};
use pipe::{DepthMode, Effect, NewEffect};
use sprite::{SpriteRender, SpriteSheet};
use sprite_visibility::SpriteVisibility;
use tex::Texture;
use types::{Encoder, Factory, Slice};

/// Draws sprites on a 2D quad.
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawSprite {
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl DrawSprite
where
    Self: Pass,
{
    /// Create instance of `DrawSprite` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }
}

impl<'a> PassData<'a> for DrawSprite {
    type Data = (
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AssetStorage<SpriteSheet>>,
        Read<'a, AssetStorage<Texture>>,
        Read<'a, MaterialTextureSet>,
        Option<Read<'a, SpriteVisibility>>,
        ReadStorage<'a, SpriteRender>,
        ReadStorage<'a, GlobalTransform>,
    );
}

impl Pass for DrawSprite {
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        use gfx::format::{ChannelType, Format, SurfaceType};
        use gfx::pso::buffer::Element;
        use std::mem;

        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder
            .with_raw_constant_buffer(
                "ViewArgs",
                mem::size_of::<<ViewArgs as Uniform>::Std140>(),
                1,
            )
            .with_raw_vertex_buffer(
                &[
                    (
                        "size",
                        Element {
                            offset: 0,
                            format: Format(SurfaceType::R32_G32, ChannelType::Float),
                        },
                    ),
                    (
                        "offsets",
                        Element {
                            offset: 8,
                            format: Format(SurfaceType::R32_G32, ChannelType::Float),
                        },
                    ),
                    (
                        "u_offset",
                        Element {
                            offset: 16,
                            format: Format(SurfaceType::R32_G32, ChannelType::Float),
                        },
                    ),
                    (
                        "v_offset",
                        Element {
                            offset: 24,
                            format: Format(SurfaceType::R32_G32, ChannelType::Float),
                        },
                    ),
                ],
                32,
                1,
            );
        setup_textures(&mut builder, &TEXTURES);
        match self.transparency {
            Some((mask, blend, depth)) => builder.with_blended_output("color", mask, blend, depth),
            None => builder.with_output("color", Some(DepthMode::LessEqualWrite)),
        };
        builder.build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        mut factory: Factory,
        (
            active,
            camera,
            sprite_sheet_storage,
            tex_storage,
            material_texture_set,
            visibility,
            sprite_render,
            global,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        let mut batch = SpriteBatch::new();
        match visibility {
            None => {
                for (sprite_render, global) in (&sprite_render, &global).join() {
                    batch.add_sprite(
                        sprite_render,
                        Some(global),
                        &sprite_sheet_storage,
                        &material_texture_set,
                    );
                }
                batch.sort();
            }
            Some(ref visibility) => {
                for (sprite_render, global, _) in
                    (&sprite_render, &global, &visibility.visible_unordered).join()
                {
                    batch.add_sprite(
                        sprite_render,
                        Some(global),
                        &sprite_sheet_storage,
                        &material_texture_set,
                    );
                }

                // We are free to optimize the order of the opaque sprites.
                batch.sort();

                for entity in &visibility.visible_ordered {
                    if let Some(sprite_render) = sprite_render.get(*entity) {
                        batch.add_sprite(
                            sprite_render,
                            global.get(*entity),
                            &sprite_sheet_storage,
                            &material_texture_set,
                        );
                    }
                }
            }
        }
        batch.encode(
            encoder,
            &mut factory,
            effect,
            camera,
            &sprite_sheet_storage,
            &tex_storage,
        );
    }
}

struct SpriteDrawData {
    texture: Handle<Texture>,
    render: SpriteRender,
    transform: GlobalTransform,
}

struct SpriteBatch {
    sprites: Vec<SpriteDrawData>,
}

impl SpriteBatch {
    pub fn new() -> Self {
        SpriteBatch {
            sprites: Vec::new(),
        }
    }

    pub fn add_sprite(
        &mut self,
        sprite_render: &SpriteRender,
        global: Option<&GlobalTransform>,
        sprite_sheet_storage: &AssetStorage<SpriteSheet>,
        material_texture_set: &MaterialTextureSet,
    ) {
        if global.is_none() {
            return;
        }

        let sprite_sheet = sprite_sheet_storage.get(&sprite_render.sprite_sheet);
        if sprite_sheet.is_none() {
            warn!(
                "Sprite sheet not loaded for sprite_render: `{:?}`.",
                sprite_render
            );
            return;
        }
        let sprite_sheet = sprite_sheet.unwrap();

        let texture_handle = material_texture_set.handle(sprite_sheet.texture_id);
        if texture_handle.is_none() {
            warn!(
                "Texture handle not found for texture id: `{}`.",
                sprite_sheet.texture_id
            );
            return;
        }

        self.sprites.push(SpriteDrawData {
            texture: texture_handle.unwrap(),
            render: sprite_render.clone(),
            transform: global.unwrap().clone(),
        });
    }

    /// Optimize the sprite order to generating more coherent batches.
    pub fn sort(&mut self) {
        // Only takes the texture into account for now.
        self.sprites
            .sort_by(|a, b| a.texture.id().cmp(&b.texture.id()));
    }

    pub fn encode(
        &self,
        encoder: &mut Encoder,
        factory: &mut Factory,
        effect: &mut Effect,
        camera: Option<(&Camera, &GlobalTransform)>,
        sprite_sheet_storage: &AssetStorage<SpriteSheet>,
        tex_storage: &AssetStorage<Texture>,
    ) {
        use gfx::buffer;
        use gfx::memory::{Bind, Typed};
        use gfx::Factory;

        if self.sprites.is_empty() {
            return;
        }

        // Sprite vertex shader
        set_view_args(effect, encoder, camera);

        for sprite in &self.sprites {
            let sprite_sheet = sprite_sheet_storage.get(&sprite.render.sprite_sheet);
            if sprite_sheet.is_none() {
                warn!(
                    "Sprite sheet not loaded for sprite_render: `{:?}`.",
                    sprite.render
                );
                return;
            }
            let sprite_sheet = sprite_sheet.unwrap();

            let texture = tex_storage.get(&sprite.texture);
            if texture.is_none() {
                warn!(
                    "Texture not loaded for texture id: `{}`.",
                    sprite_sheet.texture_id
                );
                return;
            }
            add_texture(effect, texture.unwrap());

            let sprite_data = &sprite_sheet.sprites[sprite.render.sprite_number];

            let tex_coords = &sprite_data.tex_coords;
            let (uv_left, uv_right) = if sprite.render.flip_horizontal {
                (tex_coords.right, tex_coords.left)
            } else {
                (tex_coords.left, tex_coords.right)
            };
            let (uv_bottom, uv_top) = if sprite.render.flip_vertical {
                (tex_coords.top, tex_coords.bottom)
            } else {
                (tex_coords.bottom, tex_coords.top)
            };

            let transform = &sprite.transform.0;

            let size = transform * Vector4::new(sprite_data.width, sprite_data.height, 0.0, 0.0);
            let offset =
                transform * Vector4::new(sprite_data.offsets[0], sprite_data.offsets[1], 0.0, 1.0);

            let vbuf = factory
                .create_buffer_immutable(
                    &[
                        size.x, size.y, offset.x, offset.y, uv_left, uv_right, uv_bottom, uv_top,
                    ],
                    buffer::Role::Vertex,
                    Bind::empty(),
                )
                .unwrap();

            effect.data.vertex_bufs.push(vbuf.raw().clone());
            effect.data.vertex_bufs.push(vbuf.raw().clone());
            effect.data.vertex_bufs.push(vbuf.raw().clone());
            effect.data.vertex_bufs.push(vbuf.raw().clone());

            effect.draw(
                &Slice {
                    start: 0,
                    end: 6,
                    base_vertex: 0,
                    instances: None,
                    buffer: Default::default(),
                },
                encoder,
            );
            effect.clear();
        }
    }
}

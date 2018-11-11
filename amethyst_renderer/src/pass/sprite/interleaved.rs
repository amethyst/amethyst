//! Flat forward drawing pass that mimics a blit.

use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    nalgebra::Vector4,
    specs::prelude::{Join, Read, ReadStorage},
    transform::GlobalTransform,
};

use {
    cam::{ActiveCamera, Camera},
    error::Result,
    hidden::{Hidden, HiddenPropagate},
    mtl::MaterialTextureSet,
    pass::util::{add_texture, get_camera, set_view_args, setup_textures, ViewArgs},
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    sprite::{SpriteRender, SpriteSheet},
    sprite_visibility::SpriteVisibility,
    tex::Texture,
    types::{Encoder, Factory, Slice},
    vertex::{Attributes, Query, VertexFormat},
};

use super::*;

/// Draws sprites on a 2D quad.
#[derive(Derivative, Clone, Debug)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawSprite {
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
    batch: SpriteBatch,
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

    fn attributes() -> Attributes<'static> {
        <SpriteInstance as Query<(DirX, DirY, Pos, OffsetU, OffsetV, Depth)>>::QUERIED_ATTRIBUTES
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
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        ReadStorage<'a, SpriteRender>,
        ReadStorage<'a, GlobalTransform>,
    );
}

impl Pass for DrawSprite {
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        use std::mem;

        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder
            .without_back_face_culling()
            .with_raw_constant_buffer(
                "ViewArgs",
                mem::size_of::<<ViewArgs as Uniform>::Std140>(),
                1,
            ).with_raw_vertex_buffer(Self::attributes(), SpriteInstance::size() as ElemStride, 1);
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
            hidden,
            hidden_prop,
            sprite_render,
            global,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        match visibility {
            None => {
                for (sprite_render, global, _, _) in
                    (&sprite_render, &global, !&hidden, !&hidden_prop).join()
                {
                    self.batch.add_sprite(
                        sprite_render,
                        Some(global),
                        &sprite_sheet_storage,
                        &material_texture_set,
                        &tex_storage,
                    );
                }
                self.batch.sort();
            }
            Some(ref visibility) => {
                for (sprite_render, global, _) in
                    (&sprite_render, &global, &visibility.visible_unordered).join()
                {
                    self.batch.add_sprite(
                        sprite_render,
                        Some(global),
                        &sprite_sheet_storage,
                        &material_texture_set,
                        &tex_storage,
                    );
                }

                // We are free to optimize the order of the opaque sprites.
                self.batch.sort();

                for entity in &visibility.visible_ordered {
                    if let Some(sprite_render) = sprite_render.get(*entity) {
                        self.batch.add_sprite(
                            sprite_render,
                            global.get(*entity),
                            &sprite_sheet_storage,
                            &material_texture_set,
                            &tex_storage,
                        );
                    }
                }
            }
        }
        self.batch.encode(
            encoder,
            &mut factory,
            effect,
            camera,
            &sprite_sheet_storage,
            &tex_storage,
        );
        self.batch.reset();
    }
}

#[derive(Clone, Debug)]
struct SpriteDrawData {
    texture: Handle<Texture>,
    render: SpriteRender,
    transform: GlobalTransform,
}

#[derive(Clone, Default, Debug)]
struct SpriteBatch {
    sprites: Vec<SpriteDrawData>,
}

impl SpriteBatch {
    pub fn add_sprite(
        &mut self,
        sprite_render: &SpriteRender,
        global: Option<&GlobalTransform>,
        sprite_sheet_storage: &AssetStorage<SpriteSheet>,
        material_texture_set: &MaterialTextureSet,
        tex_storage: &AssetStorage<Texture>,
    ) {
        let global = match global {
            Some(v) => v,
            None => return,
        };

        let texture_handle = match sprite_sheet_storage.get(&sprite_render.sprite_sheet) {
            Some(sprite_sheet) => match material_texture_set.handle(sprite_sheet.texture_id) {
                Some(texture_handle) => {
                    if tex_storage.get(&texture_handle).is_none() {
                        warn!(
                            "Texture not loaded for texture id: `{}`.",
                            sprite_sheet.texture_id
                        );
                        return;
                    }

                    texture_handle
                }
                None => {
                    warn!(
                        "Texture handle not found for texture id: `{}`.",
                        sprite_sheet.texture_id
                    );
                    return;
                }
            },
            None => {
                warn!(
                    "Sprite sheet not loaded for sprite_render: `{:?}`.",
                    sprite_render
                );
                return;
            }
        };

        self.sprites.push(SpriteDrawData {
            texture: texture_handle,
            render: sprite_render.clone(),
            transform: *global,
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
        use gfx::{
            buffer,
            memory::{Bind, Typed},
            Factory,
        };

        if self.sprites.is_empty() {
            return;
        }

        // Sprite vertex shader
        set_view_args(effect, encoder, camera);

        let mut instance_data = Vec::<f32>::new();
        let mut num_instances = 0;
        let num_sprites = self.sprites.len();

        for (i, sprite) in self.sprites.iter().enumerate() {
            // `unwrap`
            let sprite_sheet = sprite_sheet_storage
                .get(&sprite.render.sprite_sheet)
                .expect(
                    "Unreachable: Existence of sprite sheet checked when collecting the sprites",
                );

            let texture = tex_storage
                .get(&sprite.texture)
                .expect("Unable to get texture of sprite");

            // Append sprite to instance data.
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

            let dir_x = transform.column(0) * sprite_data.width;
            let dir_y = transform.column(1) * sprite_data.height;

            // The offsets are negated to shift the sprite left and down relative to the entity, in
            // regards to pivot points. This is the convention adopted in:
            //
            // * libgdx: <https://gamedev.stackexchange.com/q/22553>
            // * godot: <https://godotengine.org/qa/9784>
            let pos = transform
                * Vector4::new(-sprite_data.offsets[0], -sprite_data.offsets[1], 0.0, 1.0);

            instance_data.extend(&[
                dir_x.x, dir_x.y, dir_y.x, dir_y.y, pos.x, pos.y, uv_left, uv_right, uv_bottom,
                uv_top, pos.z,
            ]);
            num_instances += 1;

            // Need to flush outstanding draw calls due to state switch (texture).
            //
            // 1. We are at the last sprite and want to submit all pending work.
            // 2. The next sprite will use a different texture triggering a flush.
            let need_flush =
                i >= num_sprites - 1 || self.sprites[i + 1].texture.id() != sprite.texture.id();

            if need_flush {
                add_texture(effect, texture);

                let vbuf = factory
                    .create_buffer_immutable(&instance_data, buffer::Role::Vertex, Bind::empty())
                    .expect("Unable to create immutable buffer for `SpriteBatch`");

                for _ in DrawSprite::attributes() {
                    effect.data.vertex_bufs.push(vbuf.raw().clone());
                }

                effect.draw(
                    &Slice {
                        start: 0,
                        end: 6,
                        base_vertex: 0,
                        instances: Some((num_instances, 0)),
                        buffer: Default::default(),
                    },
                    encoder,
                );

                effect.clear();

                num_instances = 0;
                instance_data.clear();
            }
        }
    }

    pub fn reset(&mut self) {
        self.sprites.clear();
    }
}

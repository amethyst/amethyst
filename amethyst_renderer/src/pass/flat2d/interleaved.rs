//! Flat forward drawing pass that mimics a blit.

use derivative::Derivative;
use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use glsl_layout::Uniform;
use log::warn;

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    alga::general::SubsetOf,
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage},
    math::{convert, one, zero, Matrix4, RealField, Vector4},
    transform::Transform,
};
use amethyst_error::Error;

use crate::{
    cam::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    mesh::MeshHandle,
    pass::util::{
        add_texture, default_transparency, get_camera, set_view_args, set_view_args_screen,
        setup_textures, ViewArgs,
    },
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    resources::ScreenDimensions,
    screen_space::{ScreenSpace, ScreenSpaceSettings},
    sprite::{Flipped, SpriteRender, SpriteSheet},
    sprite_visibility::SpriteVisibility,
    tex::{Texture, TextureHandle},
    types::{Encoder, Factory, Slice},
    vertex::{Attributes, Query, VertexFormat},
    Color, Rgba,
};

use super::*;

/// Draws sprites on a 2D quad.
///
/// # Type Parameters:
///
/// * `N`: `RealBound` (f32, f64)
#[derive(Derivative, Clone, Debug)]
#[derivative(Default(bound = "Self: Pass"))]
pub struct DrawFlat2D<N>
where
    N: RealField + Default,
{
    #[derivative(Default(value = "default_transparency()"))]
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
    batch: TextureBatch<N>,
}

impl<N> DrawFlat2D<N>
where
    Self: Pass,
    N: RealField + Default,
{
    /// Create instance of `DrawFlat2D` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Transparency is enabled by default.
    /// If you pass false to this function transparency will be disabled.
    ///
    /// If you pass true and this was disabled previously default settings will be reinstated.
    /// If you pass true and this was already enabled this will do nothing.
    pub fn with_transparency(mut self, input: bool) -> Self {
        if input {
            if self.transparency.is_none() {
                self.transparency = default_transparency();
            }
        } else {
            self.transparency = None;
        }
        self
    }

    /// Set transparency settings to custom values.
    pub fn with_transparency_settings(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }

    fn attributes() -> Attributes<'static> {
        <SpriteInstance as Query<(DirX, DirY, Pos, OffsetU, OffsetV, Depth, Color)>>::QUERIED_ATTRIBUTES
    }
}

impl<'a, N: RealField + Default> PassData<'a> for DrawFlat2D<N> {
    type Data = (
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        Read<'a, AssetStorage<SpriteSheet>>,
        Read<'a, AssetStorage<Texture>>,
        Option<Read<'a, SpriteVisibility>>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        ReadStorage<'a, SpriteRender>,
        ReadStorage<'a, Transform<N>>,
        ReadStorage<'a, TextureHandle>,
        ReadStorage<'a, Flipped>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Rgba>,
        ReadStorage<'a, ScreenSpace>,
        ReadExpect<'a, ScreenDimensions>,
        Read<'a, ScreenSpaceSettings>,
    );
}

impl<N: RealField + Default + SubsetOf<f32>> Pass for DrawFlat2D<N> {
    fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect, Error> {
        use std::mem;

        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder
            .without_back_face_culling()
            .with_raw_constant_buffer(
                "ViewArgs",
                mem::size_of::<<ViewArgs as Uniform>::Std140>(),
                1,
            )
            .with_raw_vertex_buffer(Self::attributes(), SpriteInstance::size() as ElemStride, 1);
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
            visibility,
            hidden,
            hidden_prop,
            sprite_render,
            transform,
            texture_handle,
            flipped,
            mesh,
            rgba,
            screens,
            screen_dimensions,
            screen_space_settings,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &transform);

        match visibility {
            None => {
                for (sprite_render, transform, flipped, rgba, _, _, screen_maybe) in (
                    &sprite_render,
                    &transform,
                    flipped.maybe(),
                    rgba.maybe(),
                    !&hidden,
                    !&hidden_prop,
                    screens.maybe(),
                )
                    .join()
                {
                    self.batch.add_sprite(
                        sprite_render,
                        Some(transform),
                        flipped,
                        rgba,
                        &sprite_sheet_storage,
                        &tex_storage,
                        screen_maybe.is_some(),
                    );
                }

                for (image_render, transform, flipped, rgba, _, _, _, screen_maybe) in (
                    &texture_handle,
                    &transform,
                    flipped.maybe(),
                    rgba.maybe(),
                    !&hidden,
                    !&hidden_prop,
                    !&mesh,
                    screens.maybe(),
                )
                    .join()
                {
                    self.batch.add_image(
                        image_render,
                        Some(transform),
                        flipped,
                        rgba,
                        &tex_storage,
                        screen_maybe.is_some(),
                    );
                }

                self.batch.sort();
            }
            Some(ref visibility) => {
                for (sprite_render, transform, flipped, rgba, _, screen_maybe) in (
                    &sprite_render,
                    &transform,
                    flipped.maybe(),
                    rgba.maybe(),
                    &visibility.visible_unordered,
                    screens.maybe(),
                )
                    .join()
                {
                    self.batch.add_sprite(
                        sprite_render,
                        Some(transform),
                        flipped,
                        rgba,
                        &sprite_sheet_storage,
                        &tex_storage,
                        screen_maybe.is_some(),
                    );
                }

                for (image_render, transform, flipped, rgba, _, _, screen_maybe) in (
                    &texture_handle,
                    &transform,
                    flipped.maybe(),
                    rgba.maybe(),
                    &visibility.visible_unordered,
                    !&mesh,
                    screens.maybe(),
                )
                    .join()
                {
                    self.batch.add_image(
                        image_render,
                        Some(transform),
                        flipped,
                        rgba,
                        &tex_storage,
                        screen_maybe.is_some(),
                    );
                }

                // We are free to optimize the order of the opaque sprites.
                self.batch.sort();

                for entity in &visibility.visible_ordered {
                    let screen = screens.contains(*entity);
                    if let Some(sprite_render) = sprite_render.get(*entity) {
                        self.batch.add_sprite(
                            sprite_render,
                            transform.get(*entity),
                            flipped.get(*entity),
                            rgba.get(*entity),
                            &sprite_sheet_storage,
                            &tex_storage,
                            screen,
                        );
                    } else if let Some(texture_handle) = texture_handle.get(*entity) {
                        self.batch.add_image(
                            texture_handle,
                            transform.get(*entity),
                            flipped.get(*entity),
                            rgba.get(*entity),
                            &tex_storage,
                            screen,
                        )
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
            &screen_dimensions,
            &screen_space_settings,
        );
        self.batch.reset();
    }
}

#[derive(Clone, Debug)]
enum TextureDrawData<N: RealField> {
    Sprite {
        texture_handle: Handle<Texture>,
        render: SpriteRender,
        flipped: Option<Flipped>,
        rgba: Option<Rgba>,
        transform: Matrix4<N>,
        screen: bool,
    },
    Image {
        texture_handle: Handle<Texture>,
        transform: Matrix4<N>,
        flipped: Option<Flipped>,
        rgba: Option<Rgba>,
        width: usize,
        height: usize,
        screen: bool,
    },
}

impl<N: RealField> TextureDrawData<N> {
    pub fn texture_handle(&self) -> &Handle<Texture> {
        match self {
            TextureDrawData::Sprite { texture_handle, .. } => texture_handle,
            TextureDrawData::Image { texture_handle, .. } => texture_handle,
        }
    }

    pub fn tex_id(&self) -> u32 {
        match self {
            TextureDrawData::Sprite { texture_handle, .. } => texture_handle.id(),
            TextureDrawData::Image { texture_handle, .. } => texture_handle.id(),
        }
    }

    pub fn flipped(&self) -> &Option<Flipped> {
        match self {
            TextureDrawData::Sprite { flipped, .. } => flipped,
            TextureDrawData::Image { flipped, .. } => flipped,
        }
    }
}

#[derive(Clone, Default, Debug)]
struct TextureBatch<N: RealField> {
    textures: Vec<TextureDrawData<N>>,
    textures_screen: Vec<TextureDrawData<N>>,
}

impl<N: RealField + Default + SubsetOf<f32>> TextureBatch<N> {
    pub fn add_image(
        &mut self,
        texture_handle: &TextureHandle,
        transform: Option<&Transform<N>>,
        flipped: Option<&Flipped>,
        rgba: Option<&Rgba>,
        tex_storage: &AssetStorage<Texture>,
        screen: bool,
    ) {
        let transform = match transform {
            Some(v) => v,
            None => return,
        };

        let texture_dims = match tex_storage.get(&texture_handle) {
            Some(tex) => tex.size(),
            None => {
                warn!("Texture not loaded for texture: `{:?}`.", texture_handle);
                return;
            }
        };

        let data = TextureDrawData::Image {
            texture_handle: texture_handle.clone(),
            transform: *transform.global_matrix(),
            flipped: flipped.cloned(),
            rgba: rgba.cloned(),
            width: texture_dims.0,
            height: texture_dims.1,
            screen,
        };
        if screen {
            self.textures_screen.push(data);
        } else {
            self.textures.push(data);
        }
    }

    pub fn add_sprite(
        &mut self,
        sprite_render: &SpriteRender,
        transform: Option<&Transform<N>>,
        flipped: Option<&Flipped>,
        rgba: Option<&Rgba>,
        sprite_sheet_storage: &AssetStorage<SpriteSheet>,
        tex_storage: &AssetStorage<Texture>,
        screen: bool,
    ) {
        let transform = match transform {
            Some(v) => v,
            None => return,
        };

        let texture_handle = match sprite_sheet_storage.get(&sprite_render.sprite_sheet) {
            Some(sprite_sheet) => {
                if tex_storage.get(&sprite_sheet.texture).is_none() {
                    warn!(
                        "Texture not loaded for texture: `{:?}`.",
                        sprite_sheet.texture
                    );
                    return;
                }

                sprite_sheet.texture.clone()
            }
            None => {
                warn!(
                    "Sprite sheet not loaded for sprite_render: `{:?}`. \
                     Ensure that `RenderBundle::new(..).with_sprite_sheet_processor()` has been \
                     called and that the corresponding `SpriteSheet` asset has loaded \
                     successfully.",
                    sprite_render
                );
                return;
            }
        };

        let data = TextureDrawData::Sprite {
            texture_handle,
            render: sprite_render.clone(),
            flipped: flipped.cloned(),
            rgba: rgba.cloned(),
            transform: *transform.global_matrix(),
            screen,
        };

        if screen {
            self.textures_screen.push(data);
        } else {
            self.textures.push(data);
        }
    }

    /// Optimize the sprite order to generating more coherent batches.
    pub fn sort(&mut self) {
        // Only takes the texture into account for now.
        self.textures.sort_by(|a, b| a.tex_id().cmp(&b.tex_id()));
        self.textures_screen
            .sort_by(|a, b| a.tex_id().cmp(&b.tex_id()));
    }

    pub fn encode(
        &self,
        encoder: &mut Encoder,
        factory: &mut Factory,
        effect: &mut Effect,
        camera: Option<(&Camera, &Transform<N>)>,
        sprite_sheet_storage: &AssetStorage<SpriteSheet>,
        tex_storage: &AssetStorage<Texture>,
        screen_dimensions: &ScreenDimensions,
        screen_space_settings: &ScreenSpaceSettings,
    ) {
        if !self.textures.is_empty() {
            // Draw to world
            set_view_args(effect, encoder, camera);
            TextureBatch::encode_vec(
                &self.textures,
                encoder,
                factory,
                effect,
                sprite_sheet_storage,
                tex_storage,
            );
        }

        if !self.textures_screen.is_empty() {
            if let Some(depth_data) = &effect.data.out_depth {
                encoder.clear_depth(&depth_data.0, 1.0);
            }
            // Draw to screen
            set_view_args_screen(effect, encoder, screen_dimensions, screen_space_settings);
            TextureBatch::encode_vec(
                &self.textures_screen,
                encoder,
                factory,
                effect,
                sprite_sheet_storage,
                tex_storage,
            );
        }
    }

    fn encode_vec(
        textures: &Vec<TextureDrawData<N>>,
        encoder: &mut Encoder,
        factory: &mut Factory,
        effect: &mut Effect,
        sprite_sheet_storage: &AssetStorage<SpriteSheet>,
        tex_storage: &AssetStorage<Texture>,
    ) {
        use gfx::{
            buffer,
            memory::{Bind, Typed},
            Factory,
        };

        // We might be able to improve performance here if we
        // preallocate the maximum needed capacity. We need to
        // iterate over the sprites though to find out the longest
        // chain of sprites with the same texture, so we would need
        // to check if it actually results in an improvement over just
        // doing the allocations.
        let mut instance_data = Vec::<f32>::new();
        let mut num_instances = 0;
        let num_quads = textures.len();

        for (i, quad) in textures.iter().enumerate() {
            let texture = tex_storage
                .get(&quad.texture_handle())
                .expect("Unable to get texture of sprite");

            let (flip_horizontal, flip_vertical) = match quad.flipped() {
                Some(Flipped::Horizontal) => (true, false),
                Some(Flipped::Vertical) => (false, true),
                Some(Flipped::Both) => (true, true),
                _ => (false, false),
            };

            let (dir_x, dir_y, pos, uv_left, uv_right, uv_top, uv_bottom, rgba) = match quad {
                TextureDrawData::Sprite {
                    render,
                    transform,
                    rgba,
                    ..
                } => {
                    let sprite_sheet = sprite_sheet_storage
                        .get(&render.sprite_sheet)
                        .expect(
                            "Unreachable: Existence of sprite sheet checked when collecting the sprites",
                        );

                    // Append sprite to instance data.
                    let sprite_data = &sprite_sheet.sprites[render.sprite_number];

                    let tex_coords = &sprite_data.tex_coords;
                    let (uv_left, uv_right) = if flip_horizontal {
                        (tex_coords.right, tex_coords.left)
                    } else {
                        (tex_coords.left, tex_coords.right)
                    };
                    let (uv_bottom, uv_top) = if flip_vertical {
                        (tex_coords.top, tex_coords.bottom)
                    } else {
                        (tex_coords.bottom, tex_coords.top)
                    };

                    let global_matrix = convert::<Matrix4<N>, Matrix4<f32>>(*transform);

                    let dir_x = global_matrix.column(0) * sprite_data.width;
                    let dir_y = global_matrix.column(1) * sprite_data.height;

                    // The offsets are negated to shift the sprite left and down relative to the entity, in
                    // regards to pivot points. This is the convention adopted in:
                    //
                    // * libgdx: <https://gamedev.stackexchange.com/q/22553>
                    // * godot: <https://godotengine.org/qa/9784>
                    let pos = global_matrix
                        * Vector4::new(-sprite_data.offsets[0], -sprite_data.offsets[1], 0.0, 1.0);

                    (
                        dir_x, dir_y, pos, uv_left, uv_right, uv_top, uv_bottom, rgba,
                    )
                }
                TextureDrawData::Image {
                    transform,
                    width,
                    height,
                    rgba,
                    ..
                } => {
                    let (uv_left, uv_right) = if flip_horizontal {
                        (1.0, 0.0)
                    } else {
                        (0.0, 1.0)
                    };
                    let (uv_bottom, uv_top) = if flip_vertical {
                        (1.0, 0.0)
                    } else {
                        (0.0, 1.0)
                    };

                    let global_matrix = convert::<Matrix4<N>, Matrix4<f32>>(*transform);

                    let dir_x = global_matrix.column(0) * (*width as f32);
                    let dir_y = global_matrix.column(1) * (*height as f32);

                    let pos = global_matrix * Vector4::<f32>::new(one(), one(), zero(), one());

                    (
                        dir_x, dir_y, pos, uv_left, uv_right, uv_top, uv_bottom, rgba,
                    )
                }
            };
            let rgba = rgba.unwrap_or(Rgba::WHITE);
            instance_data.extend(&[
                dir_x.x, dir_x.y, dir_y.x, dir_y.y, pos.x, pos.y, uv_left, uv_right, uv_bottom,
                uv_top, pos.z, rgba.0, rgba.1, rgba.2, rgba.3,
            ]);
            num_instances += 1;

            // Need to flush outstanding draw calls due to state switch (texture).
            //
            // 1. We are at the last sprite and want to submit all pending work.
            // 2. The next sprite will use a different texture triggering a flush.
            let need_flush = i >= num_quads - 1
                || textures[i + 1].texture_handle().id() != quad.texture_handle().id();

            if need_flush {
                add_texture(effect, texture);

                let vbuf = factory
                    .create_buffer_immutable(&instance_data, buffer::Role::Vertex, Bind::empty())
                    .expect("Unable to create immutable buffer for `TextureBatch`");

                for _ in DrawFlat2D::<N>::attributes() {
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
        self.textures.clear();
        self.textures_screen.clear();
    }
}

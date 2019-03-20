use std::ptr;

use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::ecs::{Read, ReadExpect};
use derivative::Derivative;
use gfx::format::ChannelType;
use integer_sqrt::IntegerSquareRoot;
use shred_derive::SystemData;

use crate::{
    Sprite, SpriteRender, SpriteSheet, SpriteSheetGen, SurfaceType, Texture, TextureData,
    TextureMetadata,
};

/// Parameters to generating a colour sprite sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColourSpriteSheetParams {
    /// Individual sprite width.
    pub sprite_w: u32,
    /// Individual sprite height.
    pub sprite_h: u32,
    /// Whether there is a 1 pixel layer of padding pixels between each sprite.
    pub padded: bool,
    /// Number of rows of sprites (count vertically).
    pub row_count: usize,
    /// Number of rows of sprites (count horizontally).
    pub column_count: usize,
}

/// System data needed to load colour sprites.
#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct ColourSpriteSheetGenData<'s> {
    /// Asset `Loader`.
    #[derivative(Debug = "ignore")]
    pub loader: ReadExpect<'s, Loader>,
    /// `Texture` assets.
    #[derivative(Debug = "ignore")]
    pub texture_assets: Read<'s, AssetStorage<Texture>>,
    /// `SpriteSheet` assets.
    #[derivative(Debug = "ignore")]
    pub sprite_sheet_assets: Read<'s, AssetStorage<SpriteSheet>>,
}

const COLOUR_TRANSPARENT: [f32; 4] = [0.; 4];
/// 4 channels per pixel
const PIXEL_WIDTH: usize = 4;

/// Generates solid colour `Texture`s and `SpriteSheet`s.
#[derive(Debug)]
pub struct ColourSpriteSheetGen;

impl ColourSpriteSheetGen {
    /// Returns a `SpriteRender` that represents a single pixel sprite with the given colour.
    ///
    /// # Parameters
    ///
    /// * `colour_sprite_gen_data`: System data needed to load colour sprites.
    /// * `colour`: The colour's RGBA values, each between `0.` and `1.`.
    pub fn solid(
        ColourSpriteSheetGenData {
            loader,
            texture_assets,
            sprite_sheet_assets,
        }: &ColourSpriteSheetGenData<'_>,
        colour: [f32; 4],
    ) -> SpriteRender {
        let sprite_sheet_handle = {
            let texture_handle =
                loader.load_from_data(TextureData::from(colour), (), &texture_assets);
            let sprite = Sprite::from_pixel_values(1, 1, 1, 1, 0, 0, [0.; 2]);
            let sprites = vec![sprite];

            let sprite_sheet = SpriteSheet {
                texture: texture_handle,
                sprites,
            };

            loader.load_from_data(sprite_sheet, (), sprite_sheet_assets)
        };

        SpriteRender {
            sprite_sheet: sprite_sheet_handle,
            sprite_number: 0,
        }
    }

    /// Returns a `SpriteRender` that holds a reference to a gradient spritesheet.
    ///
    /// This generates a sprite for each colour between `colour_begin` and `colour_end` (inclusive).
    /// The number of sprites in the sprite sheet is equal to the `sprite_count` parameter.
    ///
    /// # Parameters
    ///
    /// * `colour_sprite_gen_data`: System data needed to load colour sprites.
    /// * `colour_begin`: The beginning colour's RGBA values, each between `0.` and `1.`.
    /// * `colour_end`: The ending colour's RGBA values, each between `0.` and `1.`.
    /// * `sprite_count`: Number of discreet colour sprites to generate, minimum 2.
    pub fn gradient(
        ColourSpriteSheetGenData {
            loader,
            texture_assets,
            sprite_sheet_assets,
        }: &ColourSpriteSheetGenData<'_>,
        colour_begin: [f32; 4],
        colour_end: [f32; 4],
        sprite_count: usize,
    ) -> SpriteRender {
        if sprite_count < 2 {
            panic!(
                "`sprite_count` must be at least 2, received: `{}`.",
                sprite_count
            );
        }

        let sprite_sheet_handle = {
            let column_count = sprite_count.integer_sqrt();
            let row_count = column_count + sprite_count / (column_count.pow(2) + 1);
            let params = ColourSpriteSheetParams {
                sprite_w: 1,
                sprite_h: 1,
                padded: true,
                row_count,
                column_count,
            };

            let (texture_metadata, colours) =
                Self::gradient_colours(params, colour_begin, colour_end, sprite_count);
            let (image_w, image_h) = texture_metadata
                .size
                .as_ref()
                .cloned()
                .expect("Expected `texture_metadata` image size to exist.");

            let texture_data = TextureData::F32(colours, texture_metadata);
            let texture_handle = loader.load_from_data(texture_data, (), &texture_assets);
            let sprite_sheet = SpriteSheetGen::HalfPixel.generate(
                texture_handle,
                params,
                sprite_count,
                image_w as u32,
                image_h as u32,
            );

            loader.load_from_data(sprite_sheet, (), sprite_sheet_assets)
        };

        SpriteRender {
            sprite_sheet: sprite_sheet_handle,
            sprite_number: 0,
        }
    }

    fn gradient_colours(
        ColourSpriteSheetParams {
            sprite_w,
            sprite_h,
            padded,
            row_count,
            column_count,
        }: ColourSpriteSheetParams,
        colour_begin: [f32; 4],
        colour_end: [f32; 4],
        sprite_count: usize,
    ) -> (TextureMetadata, Vec<f32>) {
        // Pixel count.
        let padding_pixels = if padded { 1 } else { 0 };
        let sprite_w_pad = sprite_w + padding_pixels;
        let sprite_h_pad = sprite_h + padding_pixels;
        let image_width = sprite_w_pad as usize * column_count;
        let image_height = sprite_h_pad as usize * row_count;
        let pixel_count = image_width * image_height;

        // Element count.
        let capacity = pixel_count * PIXEL_WIDTH;
        let mut pixel_data = vec![0f32; capacity];

        // Calculate colour values.
        //
        // Pixel coordinates are used, so Y increases downwards.

        let channel_steps =
            Self::channel_steps(sprite_count, colour_begin, colour_end, PIXEL_WIDTH);

        let row_capacity = sprite_w_pad as usize * column_count * PIXEL_WIDTH;
        (0..row_count).for_each(|sprite_row| {
            // 1. Build up a row of pixels
            // 2. Duplicate the row `sprite_h` times
            // 3. Add padding pixels if necessary
            // 4. Repeat

            let pixel_row =
                (0..column_count).fold(vec![0f32; row_capacity], |mut pixel_row, sprite_col| {
                    // For each sprite column, generate sprite_w colour pixels, and maybe
                    // 1 padding pixel.

                    let sprite_n = sprite_row * column_count + sprite_col;

                    // Calculate sprite colour
                    let sprite_colour = if sprite_n < sprite_count {
                        (0..PIXEL_WIDTH).fold(COLOUR_TRANSPARENT, |mut colour, channel| {
                            colour[channel] =
                                colour_begin[channel] + sprite_n as f32 * channel_steps[channel];
                            colour
                        })
                    } else {
                        COLOUR_TRANSPARENT
                    };

                    // Fill in `sprite_w` pixels with `sprite_colour`
                    (0..sprite_w).for_each(|pixel_n| {
                        // `pixel_n` is the pixel number, not the colour channel index in
                        // `pixel_row`.
                        let pixel_index =
                            (sprite_col * sprite_w_pad as usize + pixel_n as usize) * PIXEL_WIDTH;

                        unsafe {
                            ptr::copy_nonoverlapping(
                                sprite_colour.as_ptr(),
                                pixel_row.as_mut_ptr().offset(pixel_index as isize),
                                PIXEL_WIDTH,
                            )
                        }
                    });

                    // Not necessary to add padding pixels explicitly -- that is handled by the
                    // initialization with `capacity`.

                    pixel_row
                });

            // Copy pixel row `sprite_h` times.
            let pixel_data_row_offset = sprite_row * row_capacity * sprite_h_pad as usize;
            let pixel_row_len = pixel_row.len();
            (0..sprite_h).for_each(|pixel_row_n| unsafe {
                ptr::copy_nonoverlapping(
                    pixel_row.as_ptr(),
                    pixel_data.as_mut_ptr().offset(
                        (pixel_data_row_offset + pixel_row_n as usize * pixel_row_len) as isize,
                    ),
                    pixel_row_len,
                )
            });

            // Not necessary to add padding pixels explicitly -- that is handled by the
            // initialization with `capacity`.
        });

        let metadata = TextureMetadata::unorm()
            .with_size(image_width as u16, image_height as u16)
            .with_format(SurfaceType::R32_G32_B32_A32)
            .with_channel(ChannelType::Float);

        (metadata, pixel_data)
    }

    fn channel_steps(
        sprite_count: usize,
        colour_begin: [f32; 4],
        colour_end: [f32; 4],
        pixel_width: usize,
    ) -> [f32; 4] {
        let mut channel_steps: [f32; 4] = [0.; 4];
        for pixel_channel in 0..pixel_width {
            // Example:
            //
            // `sprite_count`: 6
            // `begin`: 3
            // `end`: 8
            //
            // Expected: 3, 4, 5, 6, 7, 8
            //
            // Step is 1, which is:
            // = 5 / 5
            // = (8 - 3) / (6 - 1)
            // = (end - start) / (sprite_count - 1)
            let channel_diff = colour_end[pixel_channel] - colour_begin[pixel_channel];
            channel_steps[pixel_channel] = channel_diff / (sprite_count - 1) as f32;
        }
        channel_steps
    }
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;

    use super::ColourSpriteSheetGen;
    use crate::ColourSpriteSheetParams;

    #[test]
    fn gradient_colours_generates_pixel_data_1x1_sprite_padded() {
        let colour_sprite_sheet_params = ColourSpriteSheetParams {
            sprite_w: 1,
            sprite_h: 1,
            padded: true,
            row_count: 2,
            column_count: 3,
        };
        let colour_begin = [1., 0.2, 0., 0.6];
        let colour_end = [0.2, 1., 0., 1.];
        let sprite_count = 5;

        let (_metadata, colours) = ColourSpriteSheetGen::gradient_colours(
            colour_sprite_sheet_params,
            colour_begin,
            colour_end,
            sprite_count,
        );

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[0..4]);
        relative_eq!([0.; 4][..], colours[4..8]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[8..12]);
        relative_eq!([0.; 4][..], colours[12..16]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[16..20]);
        relative_eq!([0.; 4][..], colours[20..24]);

        // Padding row.
        // row_length
        //     = (1 sprite_pixel + 1 padding_pixel) * column_count * 4 channels
        //     = 2 * 3 * 4
        //     = 24
        // 1 padding pixel * row_length
        relative_eq!([0.; 24][..], colours[24..48]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[48..52]);
        relative_eq!([0.; 4][..], colours[52..56]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[56..60]);
        relative_eq!([0.; 4][..], colours[60..64]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[64..68]);
        relative_eq!([0.; 4][..], colours[68..72]);

        relative_eq!([0.; 24][..], colours[72..96]);
    }

    #[test]
    fn gradient_colours_generates_pixel_data_2x1_sprite_padded() {
        let colour_sprite_sheet_params = ColourSpriteSheetParams {
            sprite_w: 2,
            sprite_h: 1,
            padded: true,
            row_count: 2,
            column_count: 3,
        };
        let colour_begin = [1., 0.2, 0., 0.6];
        let colour_end = [0.2, 1., 0., 1.];
        let sprite_count = 5;

        let (_metadata, colours) = ColourSpriteSheetGen::gradient_colours(
            colour_sprite_sheet_params,
            colour_begin,
            colour_end,
            sprite_count,
        );

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[0..4]);
        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[4..8]);
        relative_eq!([0.; 4][..], colours[8..12]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[12..16]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[16..20]);
        relative_eq!([0.; 4][..], colours[20..24]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[24..28]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[28..32]);
        relative_eq!([0.; 4][..], colours[32..36]);

        // Padding row.
        // row_length
        //     = (2 sprite_pixels + 1 padding_pixel) * column_count * 4 channels
        //     = 3 * 3 * 4
        //     = 36
        // 1 padding pixel * row_length
        relative_eq!([0.; 36][..], colours[36..72]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[72..76]);
        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[76..80]);
        relative_eq!([0.; 4][..], colours[80..84]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[84..88]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[88..92]);
        relative_eq!([0.; 4][..], colours[92..96]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[96..100]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[100..104]);
        relative_eq!([0.; 4][..], colours[104..108]);

        relative_eq!([0.; 36][..], colours[108..144]);
    }

    #[test]
    fn gradient_colours_generates_pixel_data_1x2_sprite_padded() {
        let colour_sprite_sheet_params = ColourSpriteSheetParams {
            sprite_w: 1,
            sprite_h: 2,
            padded: true,
            row_count: 2,
            column_count: 3,
        };
        let colour_begin = [1., 0.2, 0., 0.6];
        let colour_end = [0.2, 1., 0., 1.];
        let sprite_count = 5;

        let (_metadata, colours) = ColourSpriteSheetGen::gradient_colours(
            colour_sprite_sheet_params,
            colour_begin,
            colour_end,
            sprite_count,
        );

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[0..4]);
        relative_eq!([0.; 4][..], colours[4..8]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[8..12]);
        relative_eq!([0.; 4][..], colours[12..16]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[16..20]);
        relative_eq!([0.; 4][..], colours[20..24]);

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[24..28]);
        relative_eq!([0.; 4][..], colours[28..32]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[32..36]);
        relative_eq!([0.; 4][..], colours[36..40]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[40..40]);
        relative_eq!([0.; 4][..], colours[40..44]);

        // Padding row.
        // row_length
        //     = (1 sprite_pixel + 1 padding_pixel) * column_count * 4 channels
        //     = 2 * 3 * 4
        //     = 24
        // 1 padding pixel * row_length
        relative_eq!([0.; 24][..], colours[44..68]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[68..72]);
        relative_eq!([0.; 4][..], colours[72..76]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[76..80]);
        relative_eq!([0.; 4][..], colours[80..84]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[84..88]);
        relative_eq!([0.; 4][..], colours[88..92]);

        relative_eq!([0.; 24][..], colours[92..116]);
    }

    #[test]
    fn gradient_colours_generates_pixel_data_2x2_sprite_padded() {
        let colour_sprite_sheet_params = ColourSpriteSheetParams {
            sprite_w: 2,
            sprite_h: 2,
            padded: true,
            row_count: 2,
            column_count: 3,
        };
        let colour_begin = [1., 0.2, 0., 0.6];
        let colour_end = [0.2, 1., 0., 1.];
        let sprite_count = 5;

        let (_metadata, colours) = ColourSpriteSheetGen::gradient_colours(
            colour_sprite_sheet_params,
            colour_begin,
            colour_end,
            sprite_count,
        );

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[0..4]);
        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[4..8]);
        relative_eq!([0.; 4][..], colours[8..12]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[12..16]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[16..20]);
        relative_eq!([0.; 4][..], colours[20..24]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[24..28]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[28..32]);
        relative_eq!([0.; 4][..], colours[32..36]);

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[36..40]);
        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[40..44]);
        relative_eq!([0.; 4][..], colours[44..48]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[48..52]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[52..56]);
        relative_eq!([0.; 4][..], colours[56..60]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[60..64]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[64..68]);
        relative_eq!([0.; 4][..], colours[68..72]);

        // Padding row.
        // row_length
        //     = (2 sprite_pixels + 1 padding_pixel) * column_count * 4 channels
        //     = 3 * 3 * 4
        //     = 36
        // 1 padding pixel * row_length
        relative_eq!([0.; 36][..], colours[72..108]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[108..112]);
        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[112..116]);
        relative_eq!([0.; 4][..], colours[116..120]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[120..124]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[124..128]);
        relative_eq!([0.; 4][..], colours[128..132]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[132..136]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[136..140]);
        relative_eq!([0.; 4][..], colours[140..144]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[144..148]);
        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[148..152]);
        relative_eq!([0.; 4][..], colours[152..156]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[156..160]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[160..164]);
        relative_eq!([0.; 4][..], colours[164..168]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[168..172]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[172..176]);
        relative_eq!([0.; 4][..], colours[176..180]);

        relative_eq!([0.; 36][..], colours[180..216]);
    }

    #[test]
    fn gradient_colours_generates_pixel_data_1x1_sprite_unpadded() {
        let colour_sprite_sheet_params = ColourSpriteSheetParams {
            sprite_w: 1,
            sprite_h: 1,
            padded: false,
            row_count: 2,
            column_count: 3,
        };
        let colour_begin = [1., 0.2, 0., 0.6];
        let colour_end = [0.2, 1., 0., 1.];
        let sprite_count = 5;

        let (_metadata, colours) = ColourSpriteSheetGen::gradient_colours(
            colour_sprite_sheet_params,
            colour_begin,
            colour_end,
            sprite_count,
        );

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[0..4]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[4..8]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[8..12]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[12..16]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[16..20]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[20..24]);
    }

    #[test]
    fn gradient_colours_generates_pixel_data_2x1_sprite_unpadded() {
        let colour_sprite_sheet_params = ColourSpriteSheetParams {
            sprite_w: 2,
            sprite_h: 1,
            padded: false,
            row_count: 2,
            column_count: 3,
        };
        let colour_begin = [1., 0.2, 0., 0.6];
        let colour_end = [0.2, 1., 0., 1.];
        let sprite_count = 5;

        let (_metadata, colours) = ColourSpriteSheetGen::gradient_colours(
            colour_sprite_sheet_params,
            colour_begin,
            colour_end,
            sprite_count,
        );

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[0..4]);
        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[4..8]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[8..12]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[12..16]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[16..20]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[20..24]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[24..28]);
        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[28..32]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[32..36]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[36..40]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[40..44]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[44..48]);
    }

    #[test]
    fn gradient_colours_generates_pixel_data_1x2_sprite_unpadded() {
        let colour_sprite_sheet_params = ColourSpriteSheetParams {
            sprite_w: 1,
            sprite_h: 2,
            padded: false,
            row_count: 2,
            column_count: 3,
        };
        let colour_begin = [1., 0.2, 0., 0.6];
        let colour_end = [0.2, 1., 0., 1.];
        let sprite_count = 5;

        let (_metadata, colours) = ColourSpriteSheetGen::gradient_colours(
            colour_sprite_sheet_params,
            colour_begin,
            colour_end,
            sprite_count,
        );

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[0..4]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[4..8]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[8..12]);

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[12..16]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[16..20]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[20..24]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[24..28]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[28..32]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[32..36]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[36..40]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[40..44]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[44..48]);
    }

    #[test]
    fn gradient_colours_generates_pixel_data_2x2_sprite_unpadded() {
        let colour_sprite_sheet_params = ColourSpriteSheetParams {
            sprite_w: 2,
            sprite_h: 2,
            padded: false,
            row_count: 2,
            column_count: 3,
        };
        let colour_begin = [1., 0.2, 0., 0.6];
        let colour_end = [0.2, 1., 0., 1.];
        let sprite_count = 5;

        let (_metadata, colours) = ColourSpriteSheetGen::gradient_colours(
            colour_sprite_sheet_params,
            colour_begin,
            colour_end,
            sprite_count,
        );

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[0..4]);
        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[4..8]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[8..12]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[12..16]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[16..20]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[20..24]);

        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[24..28]);
        relative_eq!([1.0, 0.2, 0.0, 0.6][..], colours[28..32]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[32..36]);
        relative_eq!([0.8, 0.4, 0.0, 0.7][..], colours[36..40]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[40..44]);
        relative_eq!([0.6, 0.6, 0.0, 0.8][..], colours[44..48]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[48..52]);
        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[52..56]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[56..60]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[60..64]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[64..68]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[68..72]);

        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[72..76]);
        relative_eq!([0.4, 0.8, 0.0, 0.9][..], colours[76..80]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[80..84]);
        relative_eq!([0.2, 1.0, 0.0, 1.0][..], colours[84..88]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[88..92]);
        relative_eq!([0.0, 0.0, 0.0, 0.0][..], colours[92..96]);
    }

    #[test]
    fn channel_steps_calculates_step_correctly() {
        let sprite_count = 6;
        let colour_begin = [1., 0., 0., 0.5];
        let colour_end = [0., 1., 0., 1.];
        let pixel_width = 4;
        assert_eq!(
            [-0.2, 0.2, 0., 0.1],
            ColourSpriteSheetGen::channel_steps(
                sprite_count,
                colour_begin,
                colour_end,
                pixel_width
            )
        )
    }
}

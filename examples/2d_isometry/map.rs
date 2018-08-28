use amethyst::core::IsometricTransform;
use amethyst::ecs::World;
use amethyst::prelude::*;
use amethyst::renderer::{SpriteRender, SpriteSheetHandle, Transparent};

pub const UNIT_DIMENSIONS: (f32, f32) = (132.0, 66.0);
const MAP_SIZE: usize = 16;

#[cfg_attr(rustfmt, rustfmt_skip)]
const MAP: [[u8;MAP_SIZE];MAP_SIZE] = [
    [074, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [104, 082, 111, 067, 048, 041, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [038, 057, 057, 057, 066, 027, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [112, 067, 067, 067, 056, 049, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
    [067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067, 067],
];

pub fn initialise_map(world: &mut World, spritesheet: SpriteSheetHandle) {
    for i in 0..MAP_SIZE {
        for k in 0..MAP_SIZE {
            let mut iso_transf =
                IsometricTransform::from_unit_dimensions(UNIT_DIMENSIONS.0, UNIT_DIMENSIONS.1);
            let scale = ((UNIT_DIMENSIONS.0).powi(2) + (UNIT_DIMENSIONS.1).powi(2)).sqrt();
            iso_transf.translation.x = i as f32 * scale;
            iso_transf.translation.y = k as f32 * scale;
            world
                .create_entity()
                .with(iso_transf)
                .with(Transparent)
                .with(SpriteRender {
                    sprite_sheet: spritesheet.clone(),
                    sprite_number: MAP[i][k] as usize,
                    flip_horizontal: false,
                    flip_vertical: false,
                })
                .build();
        }
    }
}

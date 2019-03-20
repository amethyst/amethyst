use amethyst_error::Error;
use amethyst_renderer::{ColourSpriteSheetGen, ColourSpriteSheetGenData, SpriteRender};
use amethyst_test::AmethystApplication;

#[test]
fn solid_returns_sprite_render() -> Result<(), Error> {
    const RED: [f32; 4] = [1., 0.2, 0.1, 1.];

    AmethystApplication::render_base("solid_returns_sprite_render_with_colour", false)
        .with_setup(|world| {
            let sprite_render = {
                let colour_sprite_gen_data = world.system_data::<ColourSpriteSheetGenData<'_>>();
                ColourSpriteSheetGen::solid(&colour_sprite_gen_data, RED)
            };
            world.add_resource(sprite_render);
        })
        .with_assertion(|world| {
            let sprite_render = &*world.read_resource::<SpriteRender>();

            let ColourSpriteSheetGenData {
                texture_assets,
                sprite_sheet_assets,
                ..
            } = world.system_data::<ColourSpriteSheetGenData<'_>>();

            assert_eq!(0, sprite_render.sprite_number);

            let sprite_sheet = sprite_sheet_assets.get(&sprite_render.sprite_sheet);
            assert!(sprite_sheet.is_some());

            let sprite_sheet = sprite_sheet.expect("Expected `SpriteSheet` to exist.");
            assert!(texture_assets.get(&sprite_sheet.texture).is_some());
        })
        .run()
}

#[test]
fn gradient_returns_sprite_render() -> Result<(), Error> {
    const COLOUR_BEGIN: [f32; 4] = [1., 0., 0., 0.5];
    const COLOUR_END: [f32; 4] = [0., 1., 0., 1.];

    AmethystApplication::render_base("solid_returns_sprite_render_with_colour", false)
        .with_setup(|world| {
            let sprite_render = {
                let colour_sprite_gen_data = world.system_data::<ColourSpriteSheetGenData<'_>>();
                ColourSpriteSheetGen::gradient(&colour_sprite_gen_data, COLOUR_BEGIN, COLOUR_END, 5)
            };
            world.add_resource(sprite_render);
        })
        .with_assertion(|world| {
            let sprite_render = &*world.read_resource::<SpriteRender>();

            let ColourSpriteSheetGenData {
                texture_assets,
                sprite_sheet_assets,
                ..
            } = world.system_data::<ColourSpriteSheetGenData<'_>>();

            assert_eq!(0, sprite_render.sprite_number);

            let sprite_sheet = sprite_sheet_assets.get(&sprite_render.sprite_sheet);
            assert!(sprite_sheet.is_some());

            let sprite_sheet = sprite_sheet.expect("Expected `SpriteSheet` to exist.");
            assert!(texture_assets.get(&sprite_sheet.texture).is_some());
        })
        .run()
}

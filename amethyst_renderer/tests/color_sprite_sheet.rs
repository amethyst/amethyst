use amethyst_error::Error;
use amethyst_renderer::{ColorSpriteSheetGen, ColorSpriteSheetGenData, SpriteRender};
use amethyst_test::AmethystApplication;

#[test]
fn solid_returns_sprite_render() -> Result<(), Error> {
    const RED: [f32; 4] = [1., 0.2, 0.1, 1.];

    AmethystApplication::render_base("solid_returns_sprite_render", false)
        .with_setup(|world| {
            let sprite_render = {
                let color_sprite_gen_data = world.system_data::<ColorSpriteSheetGenData<'_>>();
                ColorSpriteSheetGen::solid(&color_sprite_gen_data, RED)
            };
            world.add_resource(sprite_render);
        })
        .with_assertion(|world| {
            let sprite_render = &*world.read_resource::<SpriteRender>();

            let ColorSpriteSheetGenData {
                texture_assets,
                sprite_sheet_assets,
                ..
            } = world.system_data::<ColorSpriteSheetGenData<'_>>();

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
    const COLOR_BEGIN: [f32; 4] = [1., 0., 0., 0.5];
    const COLOR_END: [f32; 4] = [0., 1., 0., 1.];

    AmethystApplication::render_base("gradient_returns_sprite_render", false)
        .with_setup(|world| {
            let sprite_render = {
                let color_sprite_gen_data = world.system_data::<ColorSpriteSheetGenData<'_>>();
                ColorSpriteSheetGen::gradient(&color_sprite_gen_data, COLOR_BEGIN, COLOR_END, 5)
            };
            world.add_resource(sprite_render);
        })
        .with_assertion(|world| {
            let sprite_render = &*world.read_resource::<SpriteRender>();

            let ColorSpriteSheetGenData {
                texture_assets,
                sprite_sheet_assets,
                ..
            } = world.system_data::<ColorSpriteSheetGenData<'_>>();

            assert_eq!(0, sprite_render.sprite_number);

            let sprite_sheet = sprite_sheet_assets.get(&sprite_render.sprite_sheet);
            assert!(sprite_sheet.is_some());

            let sprite_sheet = sprite_sheet.expect("Expected `SpriteSheet` to exist.");
            assert!(texture_assets.get(&sprite_sheet.texture).is_some());
        })
        .run()
}

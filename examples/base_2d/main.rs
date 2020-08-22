//! Custom Render Pass example

mod blend_sprite;
mod color_replace;
mod fire_sprite;
mod scrolling_sprite;

use crate::blend_sprite::*;
use crate::color_replace::*;
use crate::fire_sprite::*;
use crate::scrolling_sprite::*;

use amethyst::{
    core::transform::Transform,
    prelude::*,
    renderer::{plugins::RenderToWindow, types::DefaultBackend, RenderingBundle},
    utils::application_root_dir,
};
use amethyst_rendy::{
    Camera, ImageFormat, RenderFlat2D, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
};

use amethyst_assets::{AssetStorage, Handle, Loader};
use amethyst_core::TransformBundle;
use amethyst_rendy::resources::Tint;
use palette::Srgba;

pub struct CustomShaderState;

impl SimpleState for CustomShaderState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        //load textures
        initialise_camera(world);
        let (flag_sheet, noise_sheet) = load_sprite_sheet(world);

        //Load Sprites
        let mut transform = Transform::default();
        transform.set_translation_xyz(-120.0, 80.0, 0.0);

        world
            .create_entity()
            .with(SpriteRender {
                sprite_sheet: flag_sheet.clone(),
                sprite_number: 0,
            })
            .with(transform)
            .build();

        let mut transform = Transform::default();
        transform.set_translation_xyz(-40.0, 80.0, 0.0);

        world
            .create_entity()
            .with(ScrollingSprite {
                sprite: SpriteRender {
                    sprite_sheet: flag_sheet.clone(),
                    sprite_number: 0,
                },
            })
            .with(transform)
            .build();

        let mut transform = Transform::default();
        transform.set_translation_xyz(40.0, 80.0, 0.0);

        world
            .create_entity()
            .with(ColorReplacementSprite {
                sprite: SpriteRender {
                    sprite_sheet: flag_sheet.clone(),
                    sprite_number: 0,
                },
            })
            .with(transform)
            .with(Tint(Srgba::new(0.0, 0.0, 0.0, 1.0)))
            .build();

        let mut transform = Transform::default();
        transform.set_translation_xyz(120.0, 80.0, 0.0);

        world
            .create_entity()
            .with(ColorReplacementSprite {
                sprite: SpriteRender {
                    sprite_sheet: flag_sheet.clone(),
                    sprite_number: 0,
                },
            })
            .with(transform)
            .with(Tint(Srgba::new(1.0, 1.0, 0.0, 1.0)))
            .build();

        let mut transform = Transform::default();
        transform.set_translation_xyz(-120.0, 00.0, 0.0);

        world
            .create_entity()
            .with(FireSprite {
                sprite: SpriteRender {
                    sprite_sheet: noise_sheet.clone(),
                    sprite_number: 0,
                },
            })
            .with(transform)
            .build();

        let mut transform = Transform::default();
        transform.set_translation_xyz(-40.0, 00.0, 0.0);

        world
            .create_entity()
            .with(BlendSprite {
                sprites: [
                    SpriteRender {
                        sprite_sheet: flag_sheet.clone(),
                        sprite_number: 0,
                    },
                    SpriteRender {
                        sprite_sheet: noise_sheet.clone(),
                        sprite_number: 0,
                    },
                ],
            })
            .with(transform)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/base_2d/config/display.ron");
    let assets_dir = app_root.join("examples/base_2d/assets/");

    let game_data = GameDataBuilder::default()
        // Add the transform bundle which handles tracking entity positions
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                // The RenderToWindow plugin provides all the scaffolding for opening a window and
                // drawing on it
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.5, 1.0, 1.0, 1.0]),
                )
                // Include all of our new passes.
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderScrollingSprite::default())
                .with_plugin(RenderColorReplacement::default())
                .with_plugin(RenderBlendSprite::default())
                .with_plugin(RenderFireSprite::default()),
        )?;

    let mut game = Application::new(assets_dir, CustomShaderState, game_data)?;
    game.run();
    Ok(())
}

///Load the sprite Sheets
fn load_sprite_sheet(world: &mut World) -> (Handle<SpriteSheet>, Handle<SpriteSheet>) {
    let flag_texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "sprite/TestFlag.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };
    let noise_texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "sprite/noiseTexture.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    (
        loader.load(
            "sprite/TestFlag.ron",
            SpriteSheetFormat(flag_texture_handle),
            (),
            &sprite_sheet_store,
        ),
        loader.load(
            "sprite/noiseTexture.ron",
            SpriteSheetFormat(noise_texture_handle),
            (),
            &sprite_sheet_store,
        ),
    )
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();

    transform.set_translation_xyz(0.0, 0.0, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(320.0, 240.0))
        .with(transform)
        .build();
}

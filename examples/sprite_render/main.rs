//! Demonstrates how to load and render sprites.
//!
//! Sprite loading code based on <https://gist.github.com/trsoluti/9cc7e1fb41635cb9ded6d240cc47869d>
//! Sprites are from <https://opengameart.org/content/bat-32x32>.

extern crate amethyst;
#[macro_use]
extern crate log;
extern crate gfx;

mod loader;
mod sprite;

use amethyst::core::cgmath::{Matrix4, Transform as CgTransform, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::Entity;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{Camera, ColorMask, DisplayConfig, DrawFlat, Pipeline, PosTex, Projection,
                         RenderBundle, ScreenDimensions, Stage};
use amethyst::ui::{DrawUi, UiBundle};
use gfx::preset::blend;

use loader::SpriteSheetLoader;

const BACKGROUND_COLOUR: [f32; 4] = [0.0, 0.0, 0.0, 0.0]; // black

struct Example;

impl State for Example {
    fn on_start(&mut self, mut world: &mut World) {
        initialise_camera(world);

        let sprite_sheet_loader = SpriteSheetLoader;
        let sprite_sheet = sprite_sheet_loader.load(
            "texture/bat.32x32.png",
            sprite::Metadata::new(32., 32., 2, 6, false),
            &mut world,
        );

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };
        // This `Transform` moves the sprites to the middle of the window
        let mut common_transform = Transform::default();
        common_transform.translation = Vector3::new(width / 2., height / 2., 0.);

        for i in 0..sprite_sheet.sprite_meshes.len() {
            let mut sprite_transform = Transform::default();
            sprite_transform.translation = Vector3::new(0., i as f32 * 32., 0.);
            // sprite_transform.concat_self(&common_transform);
            world
                .create_entity()
                .with(sprite_sheet.sprite_meshes[i].clone())
                .with(sprite_sheet.image.clone())
                .with(sprite_transform)
                // Used by the engine to compute and store the rendered position.
                .with(GlobalTransform::default())
                .build();
        }
    }
}

/// This method initialises a camera which will view our sprite.
fn initialise_camera(world: &mut World) -> Entity {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            width,
            height,
            0.0,
        )))
        .with(GlobalTransform(Matrix4::from_translation(
            Vector3::new(0.0, 0.0, 1.0).into(),
        )))
        .build()
}

fn run() -> Result<(), amethyst::Error> {
    let path = format!(
        "{}/examples/sprite_render/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources_directory = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target(BACKGROUND_COLOUR, 1.0)
            .with_pass(DrawFlat::<PosTex>::new().with_transparency(
                ColorMask::all(),
                blend::ALPHA,
                None,
            ))
            .with_pass(DrawUi::new()),
    );

    let mut game = Application::build(resources_directory, Example)?
        // RenderBundle: gives us a window
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        // UiBundle relies on this as some Ui objects take input
        .with_bundle(InputBundle::<String, String>::new())?
        // Draws textures
        .with_bundle(UiBundle::<String, String>::new())?
        // Handles transformations of textures
        .with_bundle(TransformBundle::new())?
        .build()?;

    game.run();

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

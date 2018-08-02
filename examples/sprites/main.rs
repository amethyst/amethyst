//! Demonstrates how to load and render sprites.
//!
//! Sprites are from <https://opengameart.org/content/bat-32x32>.

extern crate amethyst;
extern crate amethyst_animation;
#[macro_use]
extern crate log;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod png_loader;
mod sprite;
mod sprite_sheet_loader;

use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::{Matrix4, Point3, Transform as CgTransform, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::Entity;
use amethyst::input::{is_close_requested, is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ColorMask, DrawSprite, Event, MaterialTextureSet, Projection, ScreenDimensions,
    SpriteRender, SpriteSheet, SpriteSheetHandle, VirtualKeyCode, ALPHA,
};
use amethyst::ui::UiBundle;

use sprite::SpriteSheetDefinition;

#[derive(Debug, Default)]
struct Example {
    /// The bat entities.
    entities: Vec<Entity>,
}

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        initialise_camera(world);

        let (sprite_sheet_handle, sprite_count, sprite_w, sprite_h) = load_sprite_sheet(world);

        // Calculate offset to centre all sprites
        //
        // The X offset needs to be multiplied because we are drawing the sprites across the window;
        // we don't need to multiply the Y offset because we are only drawing the sprites in 1 row.
        let sprite_offset_x = sprite_count as f32 * sprite_w / 2.;
        let sprite_offset_y = sprite_h;

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };
        // This `Transform` moves the sprites to the middle of the window
        let mut common_transform = Transform::default();
        common_transform.translation = Vector3::new(
            width / 2. - sprite_offset_x,
            height / 2. - sprite_offset_y,
            0.,
        );

        self.draw_sprites_regular(
            world,
            &common_transform,
            sprite_sheet_handle.clone(),
            sprite_count,
            sprite_w,
        );
        self.draw_sprites_flipped(
            world,
            &common_transform,
            sprite_sheet_handle,
            sprite_count,
            sprite_w,
            sprite_h,
        );
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

impl Example {
    fn draw_sprites_regular(
        &mut self,
        world: &mut World,
        common_transform: &Transform,
        sprite_sheet_handle: SpriteSheetHandle,
        sprite_count: usize,
        sprite_w: f32,
    ) {
        // Create an entity per sprite.
        for i in 0..sprite_count {
            let mut sprite_transform = Transform::default();
            sprite_transform.translation = Vector3::new(i as f32 * sprite_w, 0., 0.);

            // This combines multiple `Transform`ations.
            // You need to `use amethyst::core::cgmath::Transform`;

            CgTransform::<Point3<f32>>::concat_self(&mut sprite_transform, &common_transform);

            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: i % sprite_count,
                flip_horizontal: false,
                flip_vertical: false,
            };

            debug!("sprite_render: `{:?}`", sprite_render);

            let entity = world
                .create_entity()
                // Render info of the default sprite
                .with(sprite_render)
                // Shift sprite to some part of the window
                .with(sprite_transform)
                // Used by the engine to compute and store the rendered position.
                .with(GlobalTransform::default())
                .build();

            // Store the entity
            self.entities.push(entity);
        }
    }

    fn draw_sprites_flipped(
        &mut self,
        world: &mut World,
        common_transform: &Transform,
        sprite_sheet_handle: SpriteSheetHandle,
        sprite_count: usize,
        sprite_w: f32,
        sprite_h: f32,
    ) {
        // Create an entity per sprite.
        for i in 0..sprite_count {
            let mut sprite_transform = Transform::default();
            sprite_transform.translation = Vector3::new(i as f32 * sprite_w, sprite_h * 1.5, 0.);

            CgTransform::<Point3<f32>>::concat_self(&mut sprite_transform, &common_transform);

            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: i % sprite_count,
                flip_horizontal: (i % 2) & 1 == 1,
                flip_vertical: ((i >> 1) % 2) & 1 == 1,
            };

            debug!("sprite_render: `{:?}`", sprite_render);

            let entity = world
                .create_entity()
                // Render info of the default sprite
                .with(sprite_render)
                // Shift sprite to some part of the window
                .with(sprite_transform)
                // Used by the engine to compute and store the rendered position.
                .with(GlobalTransform::default())
                .build();

            // Store the entity
            self.entities.push(entity);
        }
    }
}

/// Loads and returns a handle to a sprite sheet.
///
/// The sprite sheet consists of two parts:
///
/// * texture: the pixel data
/// * `SpriteSheet`: the layout information of the sprites on the image
fn load_sprite_sheet(world: &mut World) -> (SpriteSheetHandle, usize, f32, f32) {
    let sprite_sheet_index = 0;

    // Store texture in the world's `MaterialTextureSet` resource (singleton hash map)
    let texture = png_loader::load("texture/bat.32x32.png", world);
    world
        .write_resource::<MaterialTextureSet>()
        .insert(sprite_sheet_index, texture);

    let sprite_w = 32.;
    let sprite_h = 32.;
    let sprite_sheet_definition = SpriteSheetDefinition::new(sprite_w, sprite_h, 2, 6, false);

    let sprite_sheet = sprite_sheet_loader::load(sprite_sheet_index, &sprite_sheet_definition);
    let sprite_count = sprite_sheet.sprites.len();

    let sprite_sheet_handle = {
        let loader = world.read_resource::<Loader>();
        loader.load_from_data(
            sprite_sheet,
            (),
            &world.read_resource::<AssetStorage<SpriteSheet>>(),
        )
    };

    (sprite_sheet_handle, sprite_count, sprite_w, sprite_h)
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
            0.0, width, height, 0.0,
        )))
        .with(GlobalTransform(Matrix4::from_translation(
            Vector3::new(0.0, 0.0, 1.0).into(),
        )))
        .build()
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let path = format!(
        "{}/examples/sprites/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let assets_directory = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        // Handles transformations of textures
        .with_bundle(TransformBundle::new())?
        // RenderBundle gives us a window
        .with_basic_renderer(path, DrawSprite::new().with_transparency(ColorMask::all(), ALPHA, None), true)?
        // UiBundle relies on this as some Ui objects take input
        .with_bundle(InputBundle::<String, String>::new())?
        // Draws textures
        .with_bundle(UiBundle::<String, String>::new())?;
    let mut game = Application::new(assets_directory, Example::default(), game_data)?;
    game.run();

    Ok(())
}

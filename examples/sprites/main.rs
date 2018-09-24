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

mod animation;
mod png_loader;
mod sprite;
mod sprite_sheet_loader;

use amethyst::animation::{
    get_animation_set, AnimationBundle, AnimationCommand, AnimationControl, ControlState,
    EndControl,
};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::{Matrix4, Point3, Transform as CgTransform, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::Entity;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ColorMask, DisplayConfig, DrawSprite, MaterialTextureSet, Pipeline, Projection,
    RenderBundle, ScreenDimensions, SpriteRender, SpriteSheet, SpriteSheetHandle, SpriteSheetSet,
    Stage, ALPHA,
};
use amethyst::ui::UiBundle;
use amethyst::utils::application_root_dir;
use std::time::Duration;

use sprite::SpriteSheetDefinition;

#[derive(Debug, Default)]
struct Example {
    /// The bat entities.
    entities: Vec<Entity>,
}

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        initialise_camera(world);

        let (sprite_sheet_handle, sprite_sheet_index, sprite_count, sprite_w, sprite_h) =
            load_sprite_sheet(world);

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
            sprite_sheet_handle.clone(),
            sprite_count,
            sprite_w,
            sprite_h,
        );
        self.draw_sprites_animated(
            world,
            &common_transform,
            sprite_sheet_handle,
            sprite_sheet_index,
            sprite_count,
            sprite_w,
            sprite_h,
        );
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
            sprite_transform.translation = Vector3::new(i as f32 * sprite_w, sprite_h * 1.25, 0.);

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

    fn draw_sprites_animated(
        &mut self,
        world: &mut World,
        common_transform: &Transform,
        sprite_sheet_handle: SpriteSheetHandle,
        sprite_sheet_index: u64,
        sprite_count: usize,
        sprite_w: f32,
        sprite_h: f32,
    ) {
        // Load animations
        let grey_bat_animation = animation::grey_bat(world, sprite_sheet_index);
        let brown_bat_animation = animation::brown_bat(world, sprite_sheet_index);

        // Create an entity per sprite.
        for i in 0..sprite_count {
            let mut sprite_transform = Transform::default();
            sprite_transform.translation = Vector3::new(i as f32 * sprite_w, sprite_h * 2.5, 0.);

            CgTransform::<Point3<f32>>::concat_self(&mut sprite_transform, &common_transform);

            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: i % sprite_count,
                flip_horizontal: i >= (sprite_count >> 1),
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

            // Add the animation
            let animation = if i < (sprite_count >> 1) {
                grey_bat_animation.clone()
            } else {
                brown_bat_animation.clone()
            };

            let mut animation_control_set_storage = world.write_storage();
            let animation_set =
                get_animation_set::<u32, SpriteRender>(&mut animation_control_set_storage, entity)
                    .unwrap();

            let animation_id = 0;

            // Offset the animation:
            let animation_control = AnimationControl::new(
                animation,
                EndControl::Loop(None),
                // Offset from beginning
                ControlState::Deferred(Duration::from_millis(i as u64 * 200)),
                AnimationCommand::Start,
                1., // Rate at which the animation plays
            );
            animation_set.insert(animation_id, animation_control);

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
fn load_sprite_sheet(world: &mut World) -> (SpriteSheetHandle, u64, usize, f32, f32) {
    let sprite_sheet_index = 0;

    // Store texture in the world's `MaterialTextureSet` resource (singleton hash map)
    // This is used by the `DrawSprite` pass to look up the texture from the `SpriteSheet`
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

    // Store a reference to the sprite sheet, necessary for sprite render animation
    world
        .write_resource::<SpriteSheetSet>()
        .insert(sprite_sheet_index, sprite_sheet_handle.clone());

    (
        sprite_sheet_handle,
        sprite_sheet_index,
        sprite_count,
        sprite_w,
        sprite_h,
    )
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
        ))).with(GlobalTransform(Matrix4::from_translation(
            Vector3::new(0.0, 0.0, 1.0).into(),
        ))).build()
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let display_config = DisplayConfig::load(format!(
        "{}/examples/sprites/resources/display_config.ron",
        app_root
    ));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0., 0., 0., 1.], 1.)
            .with_pass(DrawSprite::new().with_transparency(ColorMask::all(), ALPHA, None)),
    );

    let assets_directory = format!("{}/examples/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with_bundle(AnimationBundle::<u32, SpriteRender>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(
            // Handles transformations of textures
            TransformBundle::new()
                .with_dep(&["animation_control_system", "sampler_interpolation_system"]),
        )?
        // RenderBundle gives us a window
        .with_bundle(RenderBundle::new(pipe, Some(display_config)).with_sprite_sheet_processor())?
        // UiBundle relies on this as some Ui objects take input
        .with_bundle(InputBundle::<String, String>::new())?
        // Draws textures
        .with_bundle(UiBundle::<String, String>::new())?;
    let mut game = Application::new(assets_directory, Example::default(), game_data)?;
    game.run();

    Ok(())
}

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

use std::time::Duration;

use amethyst::animation::{
    get_animation_set, AnimationBundle, AnimationCommand, AnimationControl, ControlState,
    EndControl,
};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::{Matrix4, Point3, Transform as CgTransform, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::Entity;
use amethyst::input::{is_close_requested, is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ColorMask, DrawFlat, Event, Material, MaterialDefaults, MaterialTextureSet, Mesh,
    PosTex, Projection, ScreenDimensions, VirtualKeyCode, ALPHA,
};
use amethyst::ui::UiBundle;

#[derive(Debug, Default)]
struct Example {
    /// The bat entities.
    entities: Vec<Entity>,
}

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        initialise_camera(world);

        let sprite_sheet_texture = png_loader::load("texture/bat.32x32.png", world);

        let sprite_w = 32.;
        let sprite_h = 32.;
        let sprite_sheet_definition =
            sprite::SpriteSheetDefinition::new(sprite_w, sprite_h, 2, 6, false);

        let sprite_sheet_index = 0;
        let sprite_sheet = sprite_sheet_loader::load(sprite_sheet_index, &sprite_sheet_definition);

        let sprite_sheet_material = {
            let mat_defaults = world.read_resource::<MaterialDefaults>();
            Material {
                albedo: sprite_sheet_texture.clone(),
                ..mat_defaults.0.clone()
            }
        };

        // Load animations
        let grey_bat_animation = animation::grey_bat(&sprite_sheet, world);
        let brown_bat_animation = animation::brown_bat(&sprite_sheet, world);

        // Calculate offset to centre all sprites
        //
        // The X offset needs to be multiplied because we are drawing the sprites across the window;
        // we don't need to multiply the Y offset because we are only drawing the sprites in 1 row.
        let sprite_count = sprite_sheet.sprites.len();
        let sprite_offset_x = sprite_count as f32 * sprite_w / 2.;
        let sprite_offset_y = sprite_h / 2.;

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

        // Store sprite sheet texture in the world's `MaterialTextureSet` resource (singleton hash
        // map)
        world
            .write_resource::<MaterialTextureSet>()
            .insert(sprite_sheet_index, sprite_sheet_texture);

        // Create an entity per sprite.
        for i in 0..sprite_count {
            let mut sprite_transform = Transform::default();
            sprite_transform.translation = Vector3::new(i as f32 * sprite_w, 0., 0.);

            // This combines multiple `Transform`ations.
            // You need to `use amethyst::core::cgmath::Transform`;

            CgTransform::<Point3<f32>>::concat_self(&mut sprite_transform, &common_transform);

            let mesh = {
                let loader = world.read_resource::<Loader>();
                loader.load_from_data(
                    create_mesh_vertices(sprite_w, sprite_h).into(),
                    (),
                    &world.read_resource::<AssetStorage<Mesh>>(),
                )
            };

            let animation = if i < (sprite_count >> 1) {
                grey_bat_animation.clone()
            } else {
                brown_bat_animation.clone()
            };

            let entity = world
                .create_entity()
                // The default `Material`, whose textures will be swapped based on the animation.
                .with(sprite_sheet_material.clone())
                // Shift sprite to some part of the window
                .with(sprite_transform)
                // This defines the coordinates in the world, where the sprites should be drawn
                // relative to the entity
                .with(mesh)
                // Used by the engine to compute and store the rendered position.
                .with(GlobalTransform::default())
                .build();

            // We also need to trigger the animation, not just attach it to the entity
            let mut animation_control_set_storage = world.write_storage();
            let animation_set =
                get_animation_set::<u32, Material>(&mut animation_control_set_storage, entity)
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
        .with_bundle(AnimationBundle::<u32, Material>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        // Handles transformations of textures
        .with_bundle(
            TransformBundle::new()
                .with_dep(&["animation_control_system", "sampler_interpolation_system"]),
        )?
        // RenderBundle gives us a window
        .with_basic_renderer(path, DrawFlat::<PosTex>::new().with_transparency(ColorMask::all(), ALPHA, None), true)?
        // UiBundle relies on this as some Ui objects take input
        .with_bundle(InputBundle::<String, String>::new())?
        // Draws textures
        .with_bundle(UiBundle::<String, String>::new())?;
    let mut game = Application::new(assets_directory, Example::default(), game_data)?;
    game.run();

    Ok(())
}

/// Returns a set of vertices that make up a rectangular mesh of the given size.
///
/// This function expects pixel coordinates -- starting from the top left of the image. X increases
/// to the right, Y increases downwards.
///
/// # Parameters
///
/// * `sprite_w`: Width of each sprite, excluding the border pixel if any.
/// * `sprite_h`: Height of each sprite, excluding the border pixel if any.
fn create_mesh_vertices(sprite_w: f32, sprite_h: f32) -> Vec<PosTex> {
    let tex_coord_left = 0.;
    let tex_coord_right = 1.;
    // Inverse the pixel coordinates when transforming them into texture coordinates, because the
    // render passes' Y axis is 0 from the bottom of the image, and increases to 1.0 at the top of
    // the image.
    let tex_coord_top = 0.;
    let tex_coord_bottom = 1.;

    vec![
        PosTex {
            position: [0., 0., 0.],
            tex_coord: [tex_coord_left, tex_coord_top],
        },
        PosTex {
            position: [sprite_w, 0., 0.],
            tex_coord: [tex_coord_right, tex_coord_top],
        },
        PosTex {
            position: [0., sprite_h, 0.],
            tex_coord: [tex_coord_left, tex_coord_bottom],
        },
        PosTex {
            position: [sprite_w, sprite_h, 0.],
            tex_coord: [tex_coord_right, tex_coord_bottom],
        },
        PosTex {
            position: [0., sprite_h, 0.],
            tex_coord: [tex_coord_left, tex_coord_bottom],
        },
        PosTex {
            position: [sprite_w, 0., 0.],
            tex_coord: [tex_coord_right, tex_coord_top],
        },
    ]
}

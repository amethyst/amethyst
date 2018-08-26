//! Demonstrates sprite z ordering
//!
//! Sprites are originally from <https://opengameart.org/content/bat-32x32>, edited to show
//! layering and blending.

extern crate amethyst;
#[macro_use]
extern crate log;

mod png_loader;
mod sprite;
mod sprite_sheet_loader;

use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::{Matrix4, Ortho, Point3, Transform as CgTransform, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::Entity;
use amethyst::input::{get_key, is_close_requested, is_key_down};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ColorMask, DepthMode, DisplayConfig, DrawSprite, ElementState, MaterialTextureSet,
    Pipeline, Projection, RenderBundle, ScreenDimensions, SpriteRender, SpriteSheet,
    SpriteSheetHandle, Stage, Transparent, VirtualKeyCode, ALPHA,
};

use sprite::SpriteSheetDefinition;

const SPRITE_SPACING_RATIO: f32 = 0.7;

#[derive(Debug, Clone)]
struct LoadedSpriteSheet {
    sprite_sheet_handle: SpriteSheetHandle,
    sprite_count: usize,
    sprite_w: f32,
    sprite_h: f32,
}

#[derive(Debug, Default)]
struct Example {
    /// The camera entity
    camera: Option<Entity>,
    /// The bat entities.
    entities: Vec<Entity>,
    /// Whether or not to add the transparent component to the entities
    transparent: bool,
    /// Whether or not to reverse the Z coordinates of the entities
    ///
    /// Non-reversed means left most entity has Z: 0, and Z increases by 1.0 for each entity to the
    /// right. Reversed means the right most entity has Z: 0, and Z increases by 1.0 for each entity
    /// to the left.
    reverse: bool,
    /// Information about the loaded sprite sheet.
    loaded_sprite_sheet: Option<LoadedSpriteSheet>,
    /// Z-axis position of the camera.
    ///
    /// The Z axis increases "out of the screen". The camera faces the XY plane (i.e. towards the
    /// origin).
    camera_z: f32,
    /// Depth (Z-axis distance) that the camera can see.
    ///
    /// The camera cannot see things on the limits of its view, i.e. entities with the same Z
    /// coordinate cannot be seen, and entities at `Z - camera_depth_vision` also cannot be seen.
    /// Entities with Z coordinates between these limits are visible.
    camera_depth_vision: f32,
}

impl Example {
    fn new() -> Self {
        Example {
            camera: None,
            entities: Vec::new(),
            transparent: true,
            reverse: false,
            loaded_sprite_sheet: None,
            camera_z: 0.0,
            camera_depth_vision: 0.0,
        }
    }
}

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        self.loaded_sprite_sheet = Some(load_sprite_sheet(world));

        self.initialise_camera(world);
        self.redraw_sprites(world);
    }

    fn handle_event(
        &mut self,
        mut data: StateData<GameData>,
        event: StateEvent<()>,
    ) -> SimpleTrans<'a, 'b> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            };

            match get_key(&event) {
                Some((VirtualKeyCode::T, ElementState::Pressed)) => {
                    self.transparent = !self.transparent;
                    info!(
                        "Transparent component is {}",
                        if self.transparent {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    );
                    self.redraw_sprites(&mut data.world);
                }

                Some((VirtualKeyCode::R, ElementState::Pressed)) => {
                    self.reverse = !self.reverse;
                    info!(
                        "Sprite Z order is {}",
                        if self.reverse {
                            "reversed. Right most sprite has Z: 0, increasing to the left."
                        } else {
                            "normal. Left most sprite has Z: 0, increasing to the right."
                        }
                    );
                    self.redraw_sprites(&mut data.world);
                }

                Some((VirtualKeyCode::Up, ElementState::Pressed)) => {
                    self.camera_z -= 1.0;
                    info!("Camera Z position is: {}", self.camera_z);
                    self.adjust_camera(&mut data.world);
                    self.redraw_sprites(&mut data.world);
                }

                Some((VirtualKeyCode::Down, ElementState::Pressed)) => {
                    self.camera_z += 1.0;
                    info!("Camera Z position is: {}", self.camera_z);
                    self.adjust_camera(&mut data.world);
                    self.redraw_sprites(&mut data.world);
                }

                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    self.camera_depth_vision -= 1.0;
                    info!("Camera depth vision: {}", self.camera_depth_vision);
                    self.adjust_camera(&mut data.world);
                    self.redraw_sprites(&mut data.world);
                }

                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    self.camera_depth_vision += 1.0;
                    info!("Camera depth vision: {}", self.camera_depth_vision);
                    self.adjust_camera(&mut data.world);
                    self.redraw_sprites(&mut data.world);
                }

                _ => {}
            };
        }

        Trans::None
    }
}

impl Example {
    /// This method initialises a camera which will view our sprite.
    fn initialise_camera(&mut self, world: &mut World) {
        // Position the camera. Here we translate it forward (out of the screen) far enough to view
        // all of the sprites. Note that camera_z is 12.0, whereas the furthest sprite is 11.0.
        //
        // For the depth, the additional + 1.0 is needed because the camera can see up to, but
        // excluding, entities with a Z coordinate that is `camera_z - camera_depth_vision`. The
        // additional distance means the camera can see up to just before -1.0 on the Z axis, so
        // we can view the sprite at 0.0.
        self.camera_z = self.loaded_sprite_sheet.as_ref().unwrap().sprite_count as f32;
        self.camera_depth_vision = self.camera_z + 1.0;

        self.adjust_camera(world);
    }

    fn adjust_camera(&mut self, world: &mut World) {
        if let Some(camera) = self.camera.take() {
            world
                .delete_entity(camera)
                .expect("Failed to delete camera entity.");
        }

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        let camera = world
            .create_entity()
            .with(GlobalTransform(Matrix4::from_translation(
                Vector3::new(0.0, 0.0, self.camera_z).into(),
            )))
            // Define the view that the camera can see. It makes sense to keep the `near` value as
            // 0.0, as this means it starts seeing anything that is 0 units in front of it. The
            // `far` value is the distance the camera can see facing the origin. You can use
            // `::std::f32::MAX` to indicate that it has practically unlimited depth vision.
            .with(Camera::from(Projection::Orthographic(Ortho {
                left: 0.0,
                right: width,
                top: height,
                bottom: 0.0,
                near: 0.0,
                far: self.camera_depth_vision,
            })))
            .build();

        self.camera = Some(camera);
    }

    fn redraw_sprites(&mut self, world: &mut World) {
        let &LoadedSpriteSheet {
            sprite_count,
            sprite_w,
            ..
        } = self
            .loaded_sprite_sheet
            .as_ref()
            .expect("Expected sprite sheet to be loaded.");

        // Delete any existing entities
        self.entities.drain(..).for_each(|entity| {
            world
                .delete_entity(entity)
                .expect("Failed to delete entity.")
        });

        // Calculate offset to centre all sprites
        //
        // The X offset needs to be multiplied because we are drawing the sprites across the window;
        // we don't need to multiply the Y offset because we are only drawing the sprites in 1 row.
        let sprite_offset_x = sprite_count as f32 * sprite_w * SPRITE_SPACING_RATIO / 2.;

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };
        // This `Transform` moves the sprites to the middle of the window
        let mut common_transform = Transform::default();
        common_transform.translation = Vector3::new(width / 2. - sprite_offset_x, height / 2., 0.);

        self.draw_sprites(world, &common_transform);
    }

    fn draw_sprites(&mut self, world: &mut World, common_transform: &Transform) {
        let LoadedSpriteSheet {
            sprite_sheet_handle,
            sprite_count,
            sprite_w,
            ..
        } = self
            .loaded_sprite_sheet
            .as_ref()
            .expect("Expected sprite sheet to be loaded.")
            .clone();

        // Create an entity per sprite.
        for i in 0..sprite_count {
            let mut sprite_transform = Transform::default();

            let z = if self.reverse {
                (sprite_count - i - 1) as f32
            } else {
                i as f32
            };
            sprite_transform.translation =
                Vector3::new(i as f32 * sprite_w * SPRITE_SPACING_RATIO, z, z);

            // This combines multiple `Transform`ations.
            // You need to `use amethyst::core::cgmath::Transform`;

            CgTransform::<Point3<f32>>::concat_self(&mut sprite_transform, &common_transform);

            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: i,
                flip_horizontal: false,
                flip_vertical: false,
            };

            let mut entity_builder = world
                .create_entity()
                // Render info of the default sprite
                .with(sprite_render)
                // Shift sprite to some part of the window
                .with(sprite_transform)
                // Used by the engine to compute and store the rendered position.
                .with(GlobalTransform::default());

            // The `Transparent` component indicates that the pixel color should blend instead of
            // replacing the existing drawn pixel.
            if self.transparent {
                entity_builder = entity_builder.with(Transparent);
            }

            // Store the entity
            self.entities.push(entity_builder.build());
        }
    }
}

/// Loads and returns a handle to a sprite sheet.
///
/// The sprite sheet consists of two parts:
///
/// * texture: the pixel data
/// * `SpriteSheet`: the layout information of the sprites on the image
fn load_sprite_sheet(world: &mut World) -> LoadedSpriteSheet {
    let sprite_sheet_index = 0;

    // Store texture in the world's `MaterialTextureSet` resource (singleton hash map)
    // This is used by the `DrawSprite` pass to look up the texture from the `SpriteSheet`
    let texture = png_loader::load("texture/bat_semi_transparent.png", world);
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

    LoadedSpriteSheet {
        sprite_sheet_handle,
        sprite_count,
        sprite_w,
        sprite_h,
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_config = DisplayConfig::load(format!(
        "{}/examples/sprites_ordered/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    ));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0., 0., 0., 1.], 5.)
            .with_pass(DrawSprite::new().with_transparency(
                ColorMask::all(),
                ALPHA,
                Some(DepthMode::LessEqualWrite),
            )),
    );

    let assets_directory = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderBundle::new(pipe, Some(display_config))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&["transform_system"]),
        )?;

    let mut game = Application::new(assets_directory, Example::new(), game_data)?;
    game.run();

    Ok(())
}

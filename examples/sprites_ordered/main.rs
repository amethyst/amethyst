//! Demonstrates sprite z ordering
//!
//! Sprites are originally from <https://opengameart.org/content/bat-32x32>, edited to show
//! layering and blending.

mod sprite;
mod sprite_sheet_loader;

use amethyst::{
    assets::{AssetStorage, Loader, Processor},
    core::{Hidden, Transform, TransformBundle},
    ecs::{Entity, ReadExpect, Resources, SystemData},
    input::{get_key, is_close_requested, is_key_down, ElementState},
    prelude::*,
    renderer::{
        camera::{Camera, Projection},
        formats::texture::ImageFormat,
        pass::{DrawFlat2DDesc, DrawFlat2DTransparentDesc},
        rendy::{
            factory::Factory,
            graph::{
                render::{RenderGroupDesc, SubpassBuilder},
                GraphBuilder,
            },
            hal::{format::Format, image},
        },
        sprite::{SpriteRender, SpriteSheet, SpriteSheetHandle},
        sprite_visibility::SpriteVisibilitySortingSystem,
        transparent::Transparent,
        types::DefaultBackend,
        GraphCreator, RenderingSystem, Texture,
    },
    utils::application_root_dir,
    window::{ScreenDimensions, Window, WindowBundle},
    winit::VirtualKeyCode,
};
use std::sync::Arc;

use log::info;

use crate::sprite::SpriteSheetDefinition;

const SPRITE_SPACING_RATIO: f32 = 0.7;

#[derive(Debug, Clone)]
struct LoadedSpriteSheet {
    sprite_sheet_handle: SpriteSheetHandle,
    sprite_count: u32,
    sprite_w: u32,
    sprite_h: u32,
}

#[derive(Debug, Default)]
struct Example {
    /// The camera entity
    camera: Option<Entity>,
    /// The bat entities.
    entities: Vec<Entity>,
    /// Whether or not to add the transparent component to the entities
    transparent: bool,
    /// Whether or not to add the hidden component to the entities
    hidden: bool,
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
            hidden: false,
            reverse: false,
            loaded_sprite_sheet: None,
            camera_z: 0.0,
            camera_depth_vision: 0.0,
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        self.loaded_sprite_sheet = Some(load_sprite_sheet(world));

        self.initialise_camera(world);
        self.redraw_sprites(world);
    }

    fn handle_event(
        &mut self,
        mut data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
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

                Some((VirtualKeyCode::E, ElementState::Pressed)) => {
                    self.hidden = !self.hidden;
                    info!(
                        "Sprites are {}",
                        if self.hidden { "hidden" } else { "visible" }
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

        let mut camera_transform = Transform::default();
        camera_transform.set_translation_xyz(0.0, 0.0, self.camera_z);

        let camera = world
            .create_entity()
            .with(camera_transform)
            // Define the view that the camera can see. It makes sense to keep the `near` value as
            // 0.0, as this means it starts seeing anything that is 0 units in front of it. The
            // `far` value is the distance the camera can see facing the origin.
            .with(Camera::from(Projection::orthographic(
                -width / 2.0,
                width / 2.0,
                height / 2.,
                -height / 2.0,
                0.0,
                self.camera_depth_vision,
            )))
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
        let sprite_offset_translation_x =
            (sprite_count * sprite_w) as f32 * SPRITE_SPACING_RATIO / 2.;

        // This `Transform` moves the sprites to the middle of the window
        let mut common_transform = Transform::default();
        common_transform.set_translation_xyz(-sprite_offset_translation_x, 0.0, 0.0);

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
            sprite_transform.set_translation_xyz(
                (i * sprite_w) as f32 * SPRITE_SPACING_RATIO,
                z,
                z,
            );

            // This combines multiple `Transform`ations.
            sprite_transform.concat(&common_transform);

            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: i as usize,
            };

            let mut entity_builder = world
                .create_entity()
                // Render info of the default sprite
                .with(sprite_render)
                // Shift sprite to some part of the window
                .with(sprite_transform);

            // The `Transparent` component indicates that the pixel color should blend instead of
            // replacing the existing drawn pixel.
            if self.transparent {
                entity_builder = entity_builder.with(Transparent);
            }
            if self.hidden {
                entity_builder = entity_builder.with(Hidden);
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
    let loader = world.read_resource::<Loader>();
    let texture_handle = {
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/arrow_semi_transparent.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };
    let sprite_w = 32;
    let sprite_h = 32;
    let sprite_sheet_definition = SpriteSheetDefinition::new(sprite_w, sprite_h, 2, 6, false);

    let sprite_sheet = sprite_sheet_loader::load(texture_handle, &sprite_sheet_definition);
    let sprite_count = sprite_sheet.sprites.len() as u32;

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

    let app_root = application_root_dir()?;

    let display_config_path =
        app_root.join("examples/sprites_ordered/resources/display_config.ron");

    let assets_directory = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default()
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        .with_bundle(TransformBundle::new())?
        .with(
            Processor::<SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
        )
        .with(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &["transform_system"],
        )
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            ExampleGraph::default(),
        ));

    let mut game = Application::new(assets_directory, Example::new(), game_data)?;
    game.run();

    Ok(())
}

#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    surface_format: Option<Format>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for ExampleGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;

        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);
        let surface = factory.create_surface(&window);
        // cache surface format to speed things up
        let surface_format = *self
            .surface_format
            .get_or_insert_with(|| factory.get_surface_format(&surface));
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind =
            image::Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(ClearValue::Color([0.34, 0.36, 0.52, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let sprite = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );
        let sprite_trans = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DTransparentDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let _present = graph_builder.add_node(
            PresentNode::builder(factory, surface, color)
                .with_dependency(sprite_trans)
                .with_dependency(sprite),
        );

        graph_builder
    }
}

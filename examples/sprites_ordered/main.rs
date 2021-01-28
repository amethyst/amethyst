//! Demonstrates sprite loading and z ordering
use amethyst::{
    assets::{DefaultLoader, Handle, Loader, LoaderBundle, ProcessingQueue},
    core::{
        transform::{Transform, TransformBundle},
        Hidden,
    },
    ecs::{Entity, World},
    input::{get_key, is_close_requested, is_key_down, ElementState},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        rendy::hal::command::ClearColor,
        sprite::{SpriteGrid, Sprites},
        types::DefaultBackend,
        Camera, RenderingBundle, SpriteRender, SpriteSheet, Transparent,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    winit::event::VirtualKeyCode,
};
use log::info;

const SPRITE_SPACING_RATIO: f32 = 0.7;

#[derive(Debug, Clone)]
struct LoadedSpriteSheet {
    sprite_sheet_handle: Handle<SpriteSheet>,
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
    /// Non-reversed means left most entity has Z: 0, and Z decreases by 1.0 for each entity to the
    /// right. Reversed means the right most entity has Z: 0, and Z decreases by 1.0 for each entity
    /// to the left.
    reverse: bool,
    /// Information about the loaded sprite sheet.
    loaded_sprite_sheet: Option<LoadedSpriteSheet>,
    /// Z-axis position of the camera.
    ///
    /// The Z axis increases "out of the screen" if the camera faces the XY plane (i.e. towards the
    /// origin from (0.0, 0.0, 1.0)). This is the default orientation, when no rotation is applied to the
    /// camera's transform.
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
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        self.loaded_sprite_sheet = Some(load_sprite_sheet(resources));

        self.initialize_camera();
        self.adjust_camera(world, resources);
        self.redraw_sprites(world);
    }

    fn handle_event(
        &mut self,
        mut data: StateData<'_, GameData>,
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
                            "reversed. Right most sprite has Z: 0, decreasing to the left."
                        } else {
                            "normal. Left most sprite has Z: 0, decreasing to the right."
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
                    self.camera_z += 1.0;
                    info!("Camera Z position is: {}", self.camera_z);
                    self.adjust_camera(&mut data.world, &data.resources);
                    self.redraw_sprites(&mut data.world);
                }

                Some((VirtualKeyCode::Down, ElementState::Pressed)) => {
                    self.camera_z -= 1.0;
                    info!("Camera Z position is: {}", self.camera_z);
                    self.adjust_camera(&mut data.world, &data.resources);
                    self.redraw_sprites(&mut data.world);
                }

                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    if self.camera_depth_vision >= 2.0 {
                        self.camera_depth_vision -= 1.0;
                        info!("Camera depth vision: {}", self.camera_depth_vision);
                    }
                    self.adjust_camera(&mut data.world, &data.resources);
                    self.redraw_sprites(&mut data.world);
                }

                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    self.camera_depth_vision += 1.0;
                    info!("Camera depth vision: {}", self.camera_depth_vision);
                    self.adjust_camera(&mut data.world, &data.resources);
                    self.redraw_sprites(&mut data.world);
                }

                _ => {}
            };
        }

        Trans::None
    }
}

impl Example {
    /// This method initializes a camera which will view our sprite.
    fn initialize_camera(&mut self) {
        // Position the camera. Here we translate it forward (out of the screen) far enough to view
        // all of the sprites. Note that camera_z is 1.0, whereas the furthest sprite is -11.0.
        //
        // For the depth, the additional + 1.0 is needed because the camera can see up to, but
        // excluding, entities with a Z coordinate that is `camera_z - camera_depth_vision`. The
        // additional distance means the camera can see up to just before -12.0 on the Z axis, so
        // we can view the sprite at -11.0.
        self.camera_z = 1.0;
        self.camera_depth_vision =
            self.loaded_sprite_sheet.as_ref().unwrap().sprite_count as f32 + 1.0;
    }

    fn adjust_camera(&mut self, world: &mut World, resources: &Resources) {
        if let Some(camera) = self.camera.take() {
            world.remove(camera);
        }

        let (width, height) = {
            let dim = resources.get::<ScreenDimensions>().unwrap();
            (dim.width(), dim.height())
        };

        let mut camera_transform = Transform::default();
        camera_transform.set_translation_xyz(0.0, 0.0, self.camera_z);

        let camera = world.push((
            camera_transform,
            // Define the view that the camera can see. It makes sense to keep the `near` value as
            // 0.0, as this means it starts seeing anything that is 0 units in front of it. The
            // `far` value is the distance the camera can see facing the origin.
            Camera::orthographic(
                -width / 2.0,
                width / 2.0,
                -height / 2.0,
                height / 2.0,
                0.0,
                self.camera_depth_vision,
            ),
        ));

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
            world.remove(entity);
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
                -z,
            );

            // This combines multiple `Transform`ations.
            sprite_transform.concat(&common_transform);

            let sprite_render = SpriteRender::new(sprite_sheet_handle.clone(), i as usize);

            let sprite = world.push((
                sprite_render,
                // Shift sprite to some part of the window
                sprite_transform,
            ));
            let mut entry = world.entry(sprite).unwrap();

            // The `Transparent` component indicates that the pixel color should blend instead of
            // replacing the existing drawn pixel.
            if self.transparent {
                entry.add_component(Transparent);
            }
            if self.hidden {
                entry.add_component(Hidden);
            }

            self.entities.push(sprite);
        }
    }
}

/// Loads and returns a handle to a sprite sheet.
///
/// The sprite sheet consists of two parts:
///
/// * texture: the pixel data
/// * `SpriteSheet`: the layout information of the sprites on the image
fn load_sprite_sheet(resources: &Resources) -> LoadedSpriteSheet {
    let loader = resources.get::<DefaultLoader>().unwrap();
    let texture_handle = { loader.load("texture/arrow_semi_transparent.png") };
    let sprite_w = 32;
    let sprite_h = 32;

    let grid = SpriteGrid {
        texture_width: sprite_w * 6,
        texture_height: sprite_h * 2,
        columns: 6,
        rows: None,
        sprite_count: None,
        cell_size: None,
        position: None,
    };
    let sprite_count = grid.sprite_count();

    let sprite_sheet_handle = {
        let loader = resources.get::<DefaultLoader>().unwrap();
        let sprites = loader.load_from_data(
            Sprites::Grid(grid),
            (),
            &resources.get::<ProcessingQueue<Sprites>>().unwrap(),
        );
        loader.load_from_data(
            SpriteSheet {
                sprites,
                texture: texture_handle,
            },
            (),
            &resources.get::<ProcessingQueue<SpriteSheet>>().unwrap(),
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

    let display_config_path = app_root.join("config/display.ron");

    let assets_dir = app_root.join("assets/");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                // The RenderToWindow plugin provides all the scaffolding for opening a window and
                // drawing on it
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                // RenderFlat2D plugin is used to render entities with `SpriteRender` component.
                .with_plugin(RenderFlat2D::default()),
        );

    let game = Application::new(assets_dir, Example::new(), dispatcher)?;
    game.run();

    Ok(())
}

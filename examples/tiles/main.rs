use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        geometry::Plane,
        math::{Point3, Vector2, Vector3},
        Named, Parent, Transform, TransformBundle,
    },
    ecs::{
        Component, Entities, Entity, Join, LazyUpdate, NullStorage, Read, ReadExpect, ReadStorage,
        System, WriteStorage,
    },
    input::{is_close_requested, is_key_down, InputBundle, InputHandler, StringBindings},
    prelude::*,
    renderer::{
        camera::{ActiveCamera, Camera, Projection},
        debug_drawing::DebugLinesComponent,
        formats::texture::ImageFormat,
        palette::Srgba,
        sprite::{SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle},
        transparent::Transparent,
        types::DefaultBackend,
        RenderDebugLines, RenderFlat2D, RenderToWindow, RenderingBundle, Texture,
    },
    tiles::{RenderTiles2D, Tile, TileMap},
    utils::application_root_dir,
    window::ScreenDimensions,
    winit,
};

#[derive(Default)]
struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct DrawSelectionSystem {
    start_coordinate: Option<Point3<f32>>,
}
impl<'s> System<'s> for DrawSelectionSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, ActiveCamera>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, DebugLinesComponent>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(
        &mut self,
        (entities, active_camera, dimensions, cameras, transforms, mut debug_lines, input): Self::SystemData,
    ) {
        if let Some(lines) = (&mut debug_lines).join().next() {
            lines.clear();

            if let Some(mouse_position) = input.mouse_position() {
                let mut camera_join = (&cameras, &transforms).join();
                if let Some((camera, camera_transform)) = active_camera
                    .entity
                    .and_then(|a| camera_join.get(a, &entities))
                    .or_else(|| camera_join.next())
                {
                    let action_down = input
                        .action_is_down("select")
                        .expect("selection action missing");
                    if action_down && self.start_coordinate.is_none() {
                        // Starting a new selection
                        self.start_coordinate = Some(Point3::new(
                            mouse_position.0,
                            mouse_position.1,
                            camera_transform.translation().z,
                        ));
                    } else if action_down && self.start_coordinate.is_some() {
                        // Active drag
                        let screen_dimensions =
                            Vector2::new(dimensions.width(), dimensions.height());
                        let end_coordinate = Point3::new(
                            mouse_position.0,
                            mouse_position.1,
                            camera_transform.translation().z,
                        );

                        let start_world = camera.projection().screen_to_world_point(
                            self.start_coordinate.expect("Wut?"),
                            screen_dimensions,
                            camera_transform,
                        );
                        let end_world = camera.projection().screen_to_world_point(
                            end_coordinate,
                            screen_dimensions,
                            camera_transform,
                        );
                        println!("screen_to_world_point = {:?}, {:?}", start_world, end_world);
                        let plane = Plane::with_z(0.0);
                        let start_world_plane = camera
                            .projection()
                            .screen_ray(
                                self.start_coordinate.expect("Wut?").xy(),
                                screen_dimensions,
                                camera_transform,
                            )
                            .intersect_plane(&plane);
                        let end_world_plane = camera
                            .projection()
                            .screen_ray(end_coordinate.xy(), screen_dimensions, camera_transform)
                            .intersect_plane(&plane);
                        println!("intersect = {:?}, {:?}", start_world_plane, end_world_plane);

                        println!("Drawing box @ {:?} -> {:?}", start_world, end_world);
                        lines.add_box(start_world, end_world, Srgba::new(0.5, 0.05, 0.65, 1.0));
                    } else if !action_down && self.start_coordinate.is_some() {
                        // End drag, remove
                        self.start_coordinate = None;
                    }
                }
            }
        }
    }
}

pub struct CameraSwitchSystem {
    pressed: bool,
}
impl Default for CameraSwitchSystem {
    fn default() -> Self {
        Self { pressed: false }
    }
}
impl<'s> System<'s> for CameraSwitchSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Read<'s, ActiveCamera>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Parent>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(
        &mut self,
        (entities, lazy, active_camera, dimensions, cameras, transforms, parents, input): Self::SystemData,
    ) {
        if input.action_is_down("camera_switch").unwrap() {
            self.pressed = true;
        }
        if self.pressed && !input.action_is_down("camera_switch").unwrap() {
            println!("Switch camera released");
            self.pressed = false;

            // Lazily delete the old camera
            let mut camera_join = (&entities, &cameras, &transforms, &parents).join();
            let (old_camera_entity, old_camera, _, old_parent) = active_camera
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next())
                .unwrap();
            let old_camera_entity = old_camera_entity;

            let new_parent = old_parent.entity;
            let new_camera = match old_camera.projection() {
                Projection::Orthographic(_) => {
                    Camera::standard_3d(dimensions.width(), dimensions.height())
                }
                Projection::Perspective(_) => {
                    Camera::standard_2d(dimensions.width(), dimensions.height())
                }
                Projection::CustomMatrix(_) => unimplemented!(),
            };

            lazy.exec_mut(move |w| {
                println!("Lazily switched cameras");
                let new_camera = init_camera(
                    w,
                    new_parent,
                    Transform::from(Vector3::new(0.0, 0.0, 1.1)),
                    new_camera,
                );

                w.fetch_mut::<ActiveCamera>().entity = Some(new_camera);

                w.delete_entity(old_camera_entity).unwrap();
            });
        }
    }
}

#[derive(Default)]
pub struct CameraMovementSystem;
impl<'s> System<'s> for CameraMovementSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (active_camera, entities, cameras, mut transforms, input): Self::SystemData) {
        let x_move = input.axis_value("camera_x").unwrap();
        let y_move = input.axis_value("camera_y").unwrap();
        let z_move = input.axis_value("camera_z").unwrap();
        let z_move_scale = input.axis_value("camera_scale").unwrap();

        if x_move != 0.0 || y_move != 0.0 || z_move != 0.0 || z_move_scale != 0.0 {
            let mut camera_join = (&cameras, &mut transforms).join();
            if let Some((_, camera_transform)) = active_camera
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next())
            {
                camera_transform.prepend_translation_x(x_move * 5.0);
                camera_transform.prepend_translation_y(y_move * 5.0);
                camera_transform.prepend_translation_z(z_move);

                let z_scale = 0.01 * z_move_scale;
                let scale = camera_transform.scale();
                let scale = Vector3::new(scale.x + z_scale, scale.y + z_scale, scale.z + z_scale);
                camera_transform.set_scale(scale);
            }
        }
    }
}

fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(png_path, ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

// Initialize a sprite as a reference point at a fixed location
fn init_reference_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.1);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world
        .create_entity()
        .with(transform)
        .with(sprite)
        .with(Transparent)
        .named("reference")
        .build()
}

// Initialize a sprite as a reference point
fn init_screen_reference_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(-250.0, -245.0, 0.1);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world
        .create_entity()
        .with(transform)
        .with(sprite)
        .with(Transparent)
        .named("screen_reference")
        .build()
}

fn init_player(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.1);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 1,
    };
    world
        .create_entity()
        .with(transform)
        .with(Player)
        .with(sprite)
        .with(Transparent)
        .named("player")
        .build()
}

fn init_camera(world: &mut World, parent: Entity, transform: Transform, camera: Camera) -> Entity {
    world
        .create_entity()
        .with(transform)
        .with(Parent { entity: parent })
        .with(camera)
        .named("camera")
        .build()
}

#[derive(Default, Clone)]
struct ExampleTile;
impl Tile for ExampleTile {
    fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
        Some(16)
    }
}

struct Example;
impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<Named>();
        world.register::<Player>();

        let circle_sprite_sheet_handle = load_sprite_sheet(
            world,
            "texture/Circle_Spritesheet.png",
            "texture/Circle_Spritesheet.ron",
        );

        let map_sprite_sheet_handle =
            load_sprite_sheet(world, "texture/cp437_20x20.png", "texture/cp437_20x20.ron");

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        let _reference = init_reference_sprite(world, &circle_sprite_sheet_handle);
        let player = init_player(world, &circle_sprite_sheet_handle);
        let _camera = init_camera(
            world,
            player,
            Transform::from(Vector3::new(0.0, 0.0, 1.1)),
            Camera::standard_2d(width, height),
        );
        let _reference_screen = init_screen_reference_sprite(world, &circle_sprite_sheet_handle);

        // create a test debug lines entity
        let _ = world
            .create_entity()
            .with(DebugLinesComponent::with_capacity(1))
            .build();

        let map = TileMap::<ExampleTile>::new(
            Vector3::new(48, 48, 1),
            Vector3::new(20, 20, 1),
            Some(map_sprite_sheet_handle),
        );

        let _map_entity = world
            .create_entity()
            .with(map)
            .with(Transform::default())
            .build();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, winit::VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, winit::VirtualKeyCode::Space) {
                world.exec(
                    |(named, transforms): (ReadStorage<Named>, ReadStorage<Transform>)| {
                        for (name, transform) in (&named, &transforms).join() {
                            println!("{} => {:?}", name.name, transform.translation());
                        }
                    },
                );
                Trans::None
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(Default::default())
        .level_for("amethyst_tiles", log::LevelFilter::Warn)
        .start();

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("examples/assets");
    let display_config_path = app_root.join("examples/tiles/resources/display_config.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<StringBindings>::new()
                .with_bindings_from_file("examples/tiles/resources/input.ron")?,
        )?
        .with(
            CameraSwitchSystem::default(),
            "camera_switch",
            &["input_system"],
        )
        .with(
            CameraMovementSystem::default(),
            "movement",
            &["camera_switch"],
        )
        .with(
            DrawSelectionSystem::default(),
            "DrawSelectionSystem",
            &["camera_switch"],
        )
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderTiles2D::<ExampleTile>::default())
                .with_plugin(RenderDebugLines::default()),
        )?;

    let mut game = Application::build(assets_directory, Example)?.build(game_data)?;
    game.run();
    Ok(())
}

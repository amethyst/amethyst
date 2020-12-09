use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        math::{Point3, Vector2, Vector3},
        transform::{Parent, Transform, TransformBundle},
        Named, Time,
    },
    ecs::{
        component, system, systems::CommandBuffer, world::SubWorld, Entity, IntoQuery, Resources,
        SystemBuilder,
    },
    input::{is_close_requested, is_key_down, InputBundle, InputHandler},
    prelude::*,
    renderer::{
        camera::{ActiveCamera, Camera},
        debug_drawing::DebugLinesComponent,
        formats::texture::ImageFormat,
        palette::Srgba,
        sprite::{SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle},
        transparent::Transparent,
        types::DefaultBackend,
        RenderDebugLines, RenderFlat2D, RenderToWindow, RenderingBundle, Texture,
    },
    tiles::{MortonEncoder, RenderTiles2D, Tile, TileMap},
    utils::application_root_dir,
    window::ScreenDimensions,
    winit,
};

#[derive(Default)]
struct Player;

#[derive(Default)]
pub struct DrawSelectionSystemState {
    start_coordinate: Option<Point3<f32>>,
}

#[system]
#[read_component(Entity)]
#[read_component(ActiveCamera)]
#[read_component(Camera)]
#[read_component(Transform)]
#[write_component(DebugLinesComponent)]
fn draw_selection(
    #[state] state: &mut DrawSelectionSystemState,
    #[resource] active_camera: &ActiveCamera,
    #[resource] dimensions: &ScreenDimensions,
    #[resource] input: &InputHandler,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    let mut debug_lines_query = <&mut DebugLinesComponent>::query();
    let (mut debug_lines_subworld, mut subworld) = world.split_for_query(&debug_lines_query);
    if let Some(lines) = debug_lines_query.iter_mut(&mut debug_lines_subworld).next() {
        lines.clear();

        if let Some(mouse_position) = input.mouse_position() {
            
            // Find if the active camera exists
            let mut camera_transform = active_camera
                .entity
                .as_ref()
                .and_then(|active_camera| {
                    <(Entity, &Camera, &Transform)>::query()
                        .filter(component::<Camera>())
                        .get_mut(&mut subworld, *active_camera)
                        .ok()
                        .map(|(_, c, t)| (c, t))
                })
                .or_else(|| None);

            // Return active camera or fetch first available
            let mut camera_transform = match camera_transform {
                Some(e) => Some(e),
                None => <(&Camera, &Transform)>::query()
                .iter_mut(&mut subworld)
                .nth(0)
            };

            if let Some((camera, camera_transform)) = camera_transform {
                let action_down = input
                    .action_is_down("select")
                    .expect("selection action missing");
                if action_down && state.start_coordinate.is_none() {
                    // Starting a new selection
                    state.start_coordinate = Some(Point3::new(
                        mouse_position.0,
                        mouse_position.1,
                        camera_transform.translation().z,
                    ));
                } else if action_down && state.start_coordinate.is_some() {
                    // Active drag
                    let screen_dimensions = Vector2::new(dimensions.width(), dimensions.height());
                    let end_coordinate = Point3::new(
                        mouse_position.0,
                        mouse_position.1,
                        camera_transform.translation().z,
                    );

                    let mut start_world = camera.screen_to_world_point(
                        state.start_coordinate.expect("Wut?"),
                        screen_dimensions,
                        camera_transform,
                    );
                    let mut end_world = camera.screen_to_world_point(
                        end_coordinate,
                        screen_dimensions,
                        camera_transform,
                    );
                    start_world.z = 0.9;
                    end_world.z = 0.9;

                    lines.add_box(start_world, end_world, Srgba::new(0.5, 0.05, 0.65, 1.0));
                } else if !action_down && state.start_coordinate.is_some() {
                    // End drag, remove
                    state.start_coordinate = None;
                }
            }
        }
    }
}

pub struct CameraSwitchSystemState {
    pressed: bool,
    perspective: bool,
}
impl Default for CameraSwitchSystemState {
    fn default() -> Self {
        Self {
            pressed: false,
            perspective: false,
        }
    }
}

#[system]
#[read_component(Entity)]
#[read_component(Camera)]
#[read_component(Transform)]
#[read_component(Parent)]
fn camera_switch(
    #[state] state: &mut CameraSwitchSystemState,
    #[resource] active_camera: &mut ActiveCamera,
    #[resource] dimensions: &ScreenDimensions,
    #[resource] input: &InputHandler,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    // if input.action_is_down("camera_switch").unwrap() {
    //     state.pressed = true;
    // }
    // if state.pressed && !input.action_is_down("camera_switch").unwrap() {
    //     state.pressed = false;

    //     // Lazily delete the old camera
    //     let mut camera_query = <(&Entity, &mut Camera, &mut Transform, &Parent)>::query();

    //     let (old_camera_entity, &mut camera, &mut transform, old_parent) = active_camera
    //         .entity
    //         .and_then(|a| camera_query.get_mut(world, a).ok())
    //         .or_else(|| camera_query.iter_mut(world).nth(0))
    //         .expect("Could not fetch entity with Camera and Transform");
    //     let old_camera_entity = old_camera_entity;

    //     let new_parent = old_parent.0;

    //     state.perspective = !state.perspective;
    //     let (new_camera, new_position) = if state.perspective {
    //         (
    //             Camera::standard_3d(dimensions.width(), dimensions.height()),
    //             Vector3::new(0.0, 0.0, 500.1),
    //         )
    //     } else {
    //         (
    //             Camera::standard_2d(dimensions.width(), dimensions.height()),
    //             Vector3::new(0.0, 0.0, 1.1),
    //         )
    //     };
    //     // @todo lazy insert new camera
    //     // let mut command_buffer = CommandBuffer::new(world);
    //     // let new_camera = command_buffer.push(init_camera(world, new_parent, Transform::from(new_position), new_camera));

    //     // active_camera.entity = Some(new_camera);

    //     // command_buffer.remove(old_camera_entity);
    //     // command_buffer.flush(&mut world);
    // }
}

#[system]
#[read_component(Entity)]
#[read_component(Camera)]
#[write_component(Transform)]
fn camera_movement(
    #[resource] active_camera: &ActiveCamera,
    #[resource] input: &InputHandler,
    world: &mut SubWorld,
    _commands: &mut CommandBuffer,
) {
    let x_move = input.axis_value("camera_x").unwrap();
    let y_move = input.axis_value("camera_y").unwrap();
    let z_move = input.axis_value("camera_z").unwrap();
    let z_move_scale = input.axis_value("camera_scale").unwrap();

    if x_move != 0.0 || y_move != 0.0 || z_move != 0.0 || z_move_scale != 0.0 {
        // Find if the active camera exists
        let mut camera_transform = active_camera
            .entity
            .as_ref()
            .and_then(|active_camera| {
                <(Entity, &mut Transform)>::query()
                    .filter(component::<Camera>())
                    .get_mut(world, *active_camera)
                    .ok()
                    .map(|(_, c)| c)
            })
            .or_else(|| None);

        // Return active camera or fetch first available
        let mut camera_transform = match camera_transform {
            Some(e) => Some(e),
            None => <(&mut Transform)>::query()
            .filter(component::<Camera>())
            .iter_mut(world)
            .nth(0)
        };
    
        if let Some(camera_transform) = camera_transform {
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

struct MapMovementSystemState {
    rotate: bool,
    translate: bool,
    vector: Vector3<f32>,
}
impl Default for MapMovementSystemState {
    fn default() -> Self {
        Self {
            rotate: false,
            translate: false,
            vector: Vector3::new(100.0, 0.0, 0.0),
        }
    }
}

#[system]
#[read_component(TileMap<ExampleTile, MortonEncoder>)]
#[write_component(Transform)]
fn map_movement(
    #[state] state: &mut MapMovementSystemState,
    #[resource] time: &Time,
    #[resource] input: &InputHandler,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    if input.action_is_down("toggle_rotation").unwrap() {
        state.rotate ^= true;
    }
    if input.action_is_down("toggle_translation").unwrap() {
        state.translate ^= true;
    }
    let mut query = <(&TileMap<ExampleTile, MortonEncoder>, &mut Transform)>::query();
    if state.rotate {
        for (_, transform) in query.iter_mut(world) {
            transform.rotate_2d(time.delta_seconds());
        }
    }
    if state.translate {
        // let mut query = <(&TileMap<ExampleTile, MortonEncoder>, &mut Transform)>::query();
        for (_, transform) in query.iter_mut(world) {
            transform.prepend_translation(state.vector * time.delta_seconds());
            if transform.translation().x > 500.0 {
                state.vector = Vector3::new(-100.0, 0.0, 0.0);
            } else if transform.translation().x < -500.0 {
                state.vector = Vector3::new(100.0, 0.0, 0.0);
            }
        }
    }
}

fn load_sprite_sheet(
    world: &mut World,
    resources: &mut Resources,
    png_path: &str,
    ron_path: &str,
) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = resources.get::<Loader>().expect("Get Loader");
        let texture_storage = resources
            .get::<AssetStorage<Texture>>()
            .expect("Get Texture AssetStorage");
        loader.load(png_path, ImageFormat::default(), (), &texture_storage)
    };
    let loader = resources.get::<Loader>().expect("Get Loader");
    let sprite_sheet_store = resources
        .get::<AssetStorage<SpriteSheet>>()
        .expect("Get SpriteSheet AssetStorage");
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
    let sprite = SpriteRender::new(sprite_sheet.clone(), 0);
    world.push((transform, sprite, Transparent, Named::new("reference")))
}

// Initialize a sprite as a reference point
fn init_screen_reference_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(-250.0, -245.0, 0.1);
    let sprite = SpriteRender::new(sprite_sheet.clone(), 0);
    world.push((
        transform,
        sprite,
        Transparent,
        Named::new("screen_reference"),
    ))
}

fn init_player(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.1);
    let sprite = SpriteRender::new(sprite_sheet.clone(), 1);
    world.push((transform, Player, sprite, Transparent, Named::new("player")))
}

fn init_camera(
    parent: Entity,
    transform: Transform,
    camera: Camera,
) -> (Transform, Parent, Camera, Named) {
    (transform, Parent(parent), camera, Named::new("camera"))
}

#[derive(Default, Clone)]
struct ExampleTile;
impl Tile for ExampleTile {
    fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
        Some(1)
    }
}

struct Example;
impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;

        let circle_sprite_sheet_handle = load_sprite_sheet(
            world,
            data.resources,
            "texture/Circle_Spritesheet.png",
            "texture/Circle_Spritesheet.ron",
        );

        let map_sprite_sheet_handle = load_sprite_sheet(
            world,
            data.resources,
            "texture/cp437_20x20.png",
            "texture/cp437_20x20.ron",
        );

        let (width, height) = {
            let dim = data
                .resources
                .get::<ScreenDimensions>()
                .expect("Read ScreenDimensions");
            (dim.width(), dim.height())
        };

        let _reference = init_reference_sprite(world, &circle_sprite_sheet_handle);
        let player = init_player(world, &circle_sprite_sheet_handle);
        let _camera = world.push(init_camera(
            player,
            Transform::from(Vector3::new(0.0, 0.0, 1.1)),
            Camera::standard_2d(width, height),
        ));

        let _reference_screen = init_screen_reference_sprite(world, &circle_sprite_sheet_handle);

        let map = TileMap::<ExampleTile, MortonEncoder>::new(
            Vector3::new(48, 48, 1),
            Vector3::new(20, 20, 1),
            Some(map_sprite_sheet_handle),
        );

        // create a test debug lines entity
        world.push((DebugLinesComponent::with_capacity(1),));
        // create entity with TileMap
        world.push((map, Transform::default()));
    }

    fn handle_event(&mut self, data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        let StateData { .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, winit::VirtualKeyCode::Escape) {
                Trans::Quit
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
    let assets_directory = app_root.join("examples/tiles/assets");
    let display_config_path = app_root.join("examples/tiles/config/display.ron");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher.add_bundle(TransformBundle);
    dispatcher
        .add_bundle(InputBundle::new().with_bindings_from_file("examples/tiles/config/input.ron")?);

    dispatcher.add_system(map_movement_system(MapMovementSystemState::default()));
    dispatcher.add_system(camera_switch_system(CameraSwitchSystemState::default()));
    dispatcher.add_system(camera_movement_system());
    dispatcher.add_system(draw_selection_system(DrawSelectionSystemState::default()));
    dispatcher.add_bundle(
        RenderingBundle::<DefaultBackend>::new()
            .with_plugin(
                RenderToWindow::from_config_path(display_config_path)?
                    .with_clear([0.34, 0.36, 0.52, 1.0]),
            )
            .with_plugin(RenderDebugLines::default())
            .with_plugin(RenderFlat2D::default())
            .with_plugin(RenderTiles2D::<ExampleTile, MortonEncoder>::default()),
    );

    let mut game = Application::build(assets_directory, Example)?.build(dispatcher)?;
    game.run();
    Ok(())
}

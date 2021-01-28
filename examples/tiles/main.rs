use amethyst::{
    assets::{DefaultLoader, Handle, Loader, LoaderBundle, ProcessingQueue},
    core::{
        math::{Point3, Vector3},
        transform::{Parent, Transform, TransformBundle},
        Named,
    },
    ecs::{Entity, Resources},
    input::{is_close_requested, is_key_down, InputBundle},
    prelude::*,
    renderer::{
        camera::{ActiveCamera, Camera},
        debug_drawing::DebugLinesComponent,
        rendy::hal::command::ClearColor,
        sprite::{SpriteRender, SpriteSheet},
        transparent::Transparent,
        types::DefaultBackend,
        RenderDebugLines, RenderFlat2D, RenderToWindow, RenderingBundle,
    },
    tiles::{DrawTiles2DBoundsCameraCulling, MortonEncoder, RenderTiles2D, Tile, TileMap},
    utils::application_root_dir,
    window::ScreenDimensions,
    winit,
};

mod systems;

#[derive(Default)]
struct Player;

fn load_sprite_sheet(
    resources: &mut Resources,
    png_path: &str,
    ron_path: &str,
) -> Handle<SpriteSheet> {
    let loader = resources.get::<DefaultLoader>().expect("Get Loader");
    let texture = loader.load(png_path);
    let sprites = loader.load(ron_path);
    let sprite_sheet_store = resources.get::<ProcessingQueue<SpriteSheet>>().unwrap();
    loader.load_from_data(SpriteSheet { texture, sprites }, (), &sprite_sheet_store)
}

// Initialize a sprite as a reference point at a fixed location
fn init_reference_sprite(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.1);
    let sprite = SpriteRender::new(sprite_sheet.clone(), 0);
    world.push((transform, sprite, Transparent, Named::new("reference")))
}

// Initialize a sprite as a reference point
fn init_screen_reference_sprite(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
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

fn init_player(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.1);
    let sprite = SpriteRender::new(sprite_sheet.clone(), 1);
    world.push((transform, Player, sprite, Transparent, Named::new("player")))
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

        let (width, height) = {
            let dim = data
                .resources
                .get::<ScreenDimensions>()
                .expect("Read ScreenDimensions");
            (dim.width(), dim.height())
        };

        let circle_sprite_sheet_handle = load_sprite_sheet(
            data.resources,
            "texture/Circle_Spritesheet.png",
            "texture/Circle_Spritesheet.ron",
        );

        init_reference_sprite(world, &circle_sprite_sheet_handle);
        init_screen_reference_sprite(world, &circle_sprite_sheet_handle);

        let player = init_player(world, &circle_sprite_sheet_handle);
        let camera = world.push((
            Named("camera".into()),
            Parent(player),
            Transform::from(Vector3::new(0.0, 0.0, 1.1)),
            Camera::standard_2d(width, height),
        ));
        data.resources.insert(ActiveCamera {
            entity: Some(camera),
        });

        let map_sprite_sheet_handle = load_sprite_sheet(
            data.resources,
            "texture/cp437_20x20.png",
            "texture/cp437_20x20.ron",
        );

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
            if is_close_requested(&event)
                || is_key_down(&event, winit::event::VirtualKeyCode::Escape)
            {
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
        .level_for("amethyst_tiles", log::LevelFilter::Debug)
        .start();

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("assets");
    let display_config_path = app_root.join("config/display.ron");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher.add_bundle(LoaderBundle);
    dispatcher.add_bundle(TransformBundle);
    dispatcher
        .add_bundle(InputBundle::new().with_bindings_from_file("examples/tiles/config/input.ron")?);

    dispatcher.add_system(systems::MapMovementSystem::default());
    dispatcher.add_system(systems::CameraSwitchSystem::default());
    dispatcher.add_system(systems::CameraMovementSystem);
    dispatcher.add_system(systems::DrawSelectionSystem::default());
    dispatcher.add_bundle(
        RenderingBundle::<DefaultBackend>::new()
            .with_plugin(
                RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                    float32: [0.34, 0.36, 0.52, 1.0],
                }),
            )
            .with_plugin(RenderDebugLines::default())
            .with_plugin(RenderFlat2D::default())
            .with_plugin(RenderTiles2D::<
                ExampleTile,
                MortonEncoder,
                DrawTiles2DBoundsCameraCulling,
            >::default()),
    );

    let game = Application::build(assets_directory, Example)?.build(dispatcher)?;
    game.run();
    Ok(())
}

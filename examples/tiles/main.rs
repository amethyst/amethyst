use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        math::{Point3, Vector3},
        transform::{Parent, Transform, TransformBundle},
        Named,
    },
    ecs::{
        Entity, Resources
    },
    input::{is_close_requested, is_key_down, InputBundle},
    prelude::*,
    renderer::{
        camera::{ActiveCamera, Camera},
        debug_drawing::DebugLinesComponent,
        formats::texture::ImageFormat,
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

mod systems;

#[derive(Default)]
struct Player;

fn load_sprite_sheet(
    _world: &mut World,
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
        let camera = world.push(init_camera(
            player,
            Transform::from(Vector3::new(0.0, 0.0, 1.1)),
            Camera::standard_2d(width, height),
        ));
        data.resources.insert(ActiveCamera {
            entity: Some(camera),
        });

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

    dispatcher.add_system(Box::new(systems::MapMovementSystem::default()));
    dispatcher.add_system(Box::new(systems::CameraSwitchSystem::default()));
    dispatcher.add_system(Box::new(systems::CameraMovementSystem));
    dispatcher.add_system(Box::new(systems::DrawSelectionSystem::default()));
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

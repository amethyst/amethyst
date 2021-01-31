use amethyst::{
    assets::{DefaultLoader, Handle, Loader, LoaderBundle, ProcessingQueue},
    core::{
        transform::{Parent, Transform, TransformBundle},
        Named,
    },
    input::{is_close_requested, is_key_down, InputBundle, InputHandler, VirtualKeyCode},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        Camera, RenderingBundle, SpriteRender, SpriteSheet, Transparent,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
};

#[derive(Default)]
struct Player;

struct MovementSystem;

impl<'s> System for MovementSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MovementSystem")
                .read_resource::<InputHandler>()
                .with_query(<(Read<Player>, Write<Transform>)>::query())
                .build(|_, world, input, query| {
                    let x_move = input.axis_value("entity_x").unwrap();
                    let y_move = input.axis_value("entity_y").unwrap();

                    for (_, transform) in query.iter_mut(world) {
                        transform.prepend_translation_x(x_move as f32 * 5.0);
                        transform.prepend_translation_y(y_move as f32 * 5.0);
                    }
                }),
        )
    }
}

fn load_sprite_sheet(resources: &Resources, png_path: &str, ron_path: &str) -> Handle<SpriteSheet> {
    let loader = resources.get::<DefaultLoader>().unwrap();

    let texture = loader.load(png_path);
    let sprites = loader.load(ron_path);

    let sprite_sheet_store = resources.get::<ProcessingQueue<SpriteSheet>>().unwrap();
    loader.load_from_data(SpriteSheet { texture, sprites }, (), &sprite_sheet_store)
}

// Initialize a background
fn init_background_sprite(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_z(-10.0);
    let sprite = SpriteRender::new(sprite_sheet.clone(), 0);
    world.push((transform, sprite, Named("background".into()), Transparent))
}

// Initialize a sprite as a reference point at a fixed location
fn init_reference_sprite(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.0);
    let sprite = SpriteRender::new(sprite_sheet.clone(), 0);
    world.push((transform, sprite, Transparent, Named("reference".into())))
}

// Initialize a sprite as a reference point
fn init_screen_reference_sprite(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(-250.0, -245.0, -11.0);
    let sprite = SpriteRender::new(sprite_sheet.clone(), 0);
    world.push((
        transform,
        sprite,
        Transparent,
        Named("screen_reference".into()),
    ))
}

fn init_player(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, -3.0);
    let sprite = SpriteRender::new(sprite_sheet.clone(), 1);
    world.push((
        transform,
        Player,
        sprite,
        Transparent,
        Named("player".into()),
    ))
}

fn initialize_camera(world: &mut World, resources: &Resources, parent: Entity) -> Entity {
    let (width, height) = {
        let dim = resources.get::<ScreenDimensions>().unwrap();
        (dim.width(), dim.height())
    };

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(5.0);

    world.push((
        camera_transform,
        Parent(parent),
        Camera::standard_2d(width, height),
        Named("camera".into()),
    ))
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;

        let circle_sprite_sheet_handle = load_sprite_sheet(
            data.resources,
            "texture/Circle_Spritesheet.png",
            "texture/Circle_Spritesheet.ron",
        );
        let background_sprite_sheet_handle = load_sprite_sheet(
            data.resources,
            "texture/Background.png",
            "texture/Background.ron",
        );

        init_background_sprite(world, &background_sprite_sheet_handle);
        init_reference_sprite(world, &circle_sprite_sheet_handle);
        let player = init_player(world, &circle_sprite_sheet_handle);
        initialize_camera(world, data.resources, player);
        init_screen_reference_sprite(world, &circle_sprite_sheet_handle);
    }

    fn handle_event(&mut self, data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::Space) {
                let mut query = <(Read<Named>, Read<Transform>)>::query();
                for (name, transform) in query.iter(world) {
                    println!("{} => {:?}", name, transform.translation());
                }
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
        .level_for("amethyst::assets", log::LevelFilter::Debug)
        .start();

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("assets");
    let display_config_path = app_root.join("config/display.ron");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_bundle(InputBundle::new().with_bindings_from_file(app_root.join("config/input.ron"))?)
        .add_system(MovementSystem)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderFlat2D::default()),
        );

    let game = Application::new(assets_directory, Example, game_data)?;
    game.run();
    Ok(())
}

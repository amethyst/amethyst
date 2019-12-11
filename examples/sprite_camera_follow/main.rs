use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{Named, Parent, Transform, TransformBundle},
    derive::SystemDesc,
    ecs::{
        Component, Entity, Join, NullStorage, Read, ReadStorage, System, SystemData, WorldExt,
        WriteStorage,
    },
    input::{
        is_close_requested, is_key_down, InputBundle, InputHandler, StringBindings, VirtualKeyCode,
    },
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        Camera, ImageFormat, RenderingBundle, SpriteRender, SpriteSheet, SpriteSheetFormat,
        Texture, Transparent,
    },
    utils::application_root_dir,
    window::{DisplayConfig, EventLoop, ScreenDimensions},
};

#[derive(Default)]
struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}

#[derive(SystemDesc)]
struct MovementSystem;

impl<'s> System<'s> for MovementSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (players, mut transforms, input): Self::SystemData) {
        let x_move = input.axis_value("entity_x").unwrap();
        let y_move = input.axis_value("entity_y").unwrap();

        for (_, transform) in (&players, &mut transforms).join() {
            transform.prepend_translation_x(x_move as f32 * 5.0);
            transform.prepend_translation_y(y_move as f32 * 5.0);
            // println!("Player = {:?}", transform);
        }
    }
}

fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> Handle<SpriteSheet> {
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

// Initialize a background
fn init_background_sprite(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_z(-10.0);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world
        .create_entity()
        .with(transform)
        .with(sprite)
        .named("background")
        .with(Transparent)
        .build()
}

// Initialize a sprite as a reference point at a fixed location
fn init_reference_sprite(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.0);
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
fn init_screen_reference_sprite(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(-250.0, -245.0, -11.0);
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

fn init_player(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, -3.0);
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

fn initialise_camera(world: &mut World, parent: Entity) -> Entity {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(5.0);

    world
        .create_entity()
        .with(camera_transform)
        .with(Parent { entity: parent })
        .with(Camera::standard_2d(width, height))
        .named("camera")
        .build()
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<Named>();

        let circle_sprite_sheet_handle = load_sprite_sheet(
            world,
            "texture/Circle_Spritesheet.png",
            "texture/Circle_Spritesheet.ron",
        );
        let background_sprite_sheet_handle =
            load_sprite_sheet(world, "texture/Background.png", "texture/Background.ron");

        let _background = init_background_sprite(world, &background_sprite_sheet_handle);
        let _reference = init_reference_sprite(world, &circle_sprite_sheet_handle);
        let player = init_player(world, &circle_sprite_sheet_handle);
        let _camera = initialise_camera(world, player);
        let _reference_screen = init_screen_reference_sprite(world, &circle_sprite_sheet_handle);
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::Space) {
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

fn main() {
    amethyst::Logger::from_config(Default::default())
        .level_for("amethyst_assets", log::LevelFilter::Debug)
        .start();

    let app_root = application_root_dir().expect("Could not create application root");
    let assets_directory = app_root.join("examples/assets");
    let display_config_path = app_root.join("examples/sprite_camera_follow/config/display.ron");
    let display_config =
        DisplayConfig::load(display_config_path).expect("Failed to load DisplayConfig");

    let event_loop = EventLoop::new();
    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())
        .expect("Could not create Bundle")
        .with_bundle(
            InputBundle::<StringBindings>::new()
                .with_bindings_from_file(
                    app_root.join("examples/sprite_camera_follow/config/input.ron"),
                )
                .expect("Could not create Bundle"),
        )
        .expect("Could not create Bundle")
        .with(MovementSystem, "movement", &[])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new(display_config, &event_loop)
                .with_plugin(RenderToWindow::new().with_clear(ClearColor {
                    float32: [0.34, 0.36, 0.52, 1.0],
                }))
                .with_plugin(RenderFlat2D::default()),
        )
        .expect("Could not create Bundle");

    let mut game = Application::new(assets_directory, Example, game_data)
        .expect("Failed to create CoreApplication");
    game.initialize();
    event_loop.run(move |event, _, control_flow| {
        #[cfg(feature = "profiler")]
        profile_scope!("run_event_loop");
        game.run_winit_loop(event, control_flow)
    })
}

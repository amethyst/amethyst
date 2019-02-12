use amethyst::{
    assets::{AssetStorage, Loader},
    core::{Parent, Transform, TransformBundle},
    ecs::{Component, Entity, Join, NullStorage, Read, ReadStorage, System, WriteStorage},
    input::{InputBundle, InputHandler},
    prelude::*,
    renderer::{
        Camera, ColorMask, DepthMode, DisplayConfig, DrawFlat2D, Pipeline, PngFormat, Projection,
        RenderBundle, SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle, Stage,
        Texture, TextureMetadata, Transparent, ALPHA,
    },
    utils::application_root_dir,
};

#[derive(Default)]
struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}

struct MovementSystem;

impl<'s> System<'s> for MovementSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (players, mut transforms, input): Self::SystemData) {
        let x_move = input.axis_value("entity_x").unwrap();
        let y_move = input.axis_value("entity_y").unwrap();

        for (_, transform) in (&players, &mut transforms).join() {
            transform.translate_x(x_move as f32 * 5.0);
            transform.translate_y(y_move as f32 * 5.0);
        }
    }
}

fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            png_path,
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat,
        texture_handle,
        (),
        &sprite_sheet_store,
    )
}

// Initialize a background
fn init_background_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_z(-10.0);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world.create_entity().with(transform).with(sprite).build()
}

// Initialize a sprite as a reference point at a fixed location
fn init_reference_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_x(100.0);
    transform.set_y(0.0);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world
        .create_entity()
        .with(transform)
        .with(sprite)
        .with(Transparent)
        .build()
}

fn init_player(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_x(0.0);
    transform.set_y(0.0);
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
        .build()
}

fn init_camera(world: &mut World, parent: Entity) {
    let mut transform = Transform::default();
    transform.set_z(1.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            -250.0, 250.0, -250.0, 250.0,
        )))
        .with(Parent { entity: parent })
        .with(transform)
        .build();
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let circle_sprite_sheet_handle =
            load_sprite_sheet(world, "Circle_Spritesheet.png", "Circle_Spritesheet.ron");
        let background_sprite_sheet_handle =
            load_sprite_sheet(world, "Background.png", "Background.ron");

        let _background = init_background_sprite(world, &background_sprite_sheet_handle);
        let _reference = init_reference_sprite(world, &circle_sprite_sheet_handle);
        let parent = init_player(world, &circle_sprite_sheet_handle);
        init_camera(world, parent);
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let root = application_root_dir()?.join("examples/sprite_camera_follow/resources");
    let config = DisplayConfig::load(root.join("display_config.ron"));
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.1, 1.0], 1.0)
            .with_pass(DrawFlat2D::new().with_transparency(
                ColorMask::all(),
                ALPHA,
                Some(DepthMode::LessEqualWrite), // Tells the pipeline to respect sprite z-depth
            )),
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(root.join("input.ron"))?,
        )?
        .with(MovementSystem, "movement", &[])
        .with_bundle(
            RenderBundle::new(pipe, Some(config))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&[]), // Let's us use the `Transparent` component
        )?;

    let mut game = Application::build(root, Example)?.build(game_data)?;
    game.run();
    Ok(())
}

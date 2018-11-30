extern crate amethyst;

use amethyst::{
    assets::{AssetStorage, Loader},
    core::{Transform, TransformBundle},
    ecs::Entity,
    prelude::*,
    renderer::{
        Camera, ColorMask, DepthMode, DisplayConfig, DrawFlat2D, Pipeline, PngFormat, Projection,
        RenderBundle, Stage, Texture, TextureHandle, TextureMetadata, ALPHA,
    },
    utils::application_root_dir,
};

struct Example;

impl<S, E> StateCallback<S, E> for Example {
    fn on_start(&mut self, world: &mut World) {
        let texture_handle = load_texture(world, "logo.png");
        let _image = init_image(world, &texture_handle);

        init_camera(world)
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let resources = format!("{}/examples/simple_image/resources", application_root_dir());
    let config = DisplayConfig::load(format!("{}/display_config.ron", resources));
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.1, 1.0], 1.0)
            .with_pass(DrawFlat2D::new().with_transparency(
                ColorMask::all(),
                ALPHA,
                Some(DepthMode::LessEqualWrite),
            )),
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?;

    let mut game = Application::build(resources)?
        .with_state((), Example)?
        .build(game_data)?;

    game.run();
    Ok(())
}

fn init_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_z(1.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            -250.0, 250.0, -250.0, 250.0,
        ))).with(transform)
        .build();
}

fn init_image(world: &mut World, texture: &TextureHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_x(0.0);
    transform.set_y(0.0);

    world
        .create_entity()
        .with(transform)
        .with(texture.clone())
        .build()
}

fn load_texture(world: &mut World, png_path: &str) -> TextureHandle {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(
        png_path,
        PngFormat,
        TextureMetadata::srgb_scale(),
        (),
        &texture_storage,
    )
}

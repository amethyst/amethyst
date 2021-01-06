//! Demonstrates how to perform raycasts with the camera to project from mouse to world coordinates.
//!
//!

use amethyst::{
    assets::{AssetStorage, DefaultLoader, Handle, Loader, Progress, ProgressCounter},
    core::{
        geometry::Plane,
        math::{coordinates::XY, Point2, Vector2, Vector3},
        transform::{Transform, TransformBundle},
        Named,
    },
    ecs::{
        DispatcherBuilder, Entity, IntoQuery, ParallelRunnable, Resources, System, SystemBuilder,
    },
    input::{InputBundle, InputHandler},
    prelude::World,
    renderer::{
        camera::{ActiveCamera, Camera},
        plugins::{RenderFlat2D, RenderToWindow},
        rendy::hal::command::ClearColor,
        sprite::{SpriteRender, SpriteSheet, Sprites},
        types::DefaultBackend,
        ImageFormat, RenderingBundle, Texture,
    },
    ui::{Anchor, RenderUi, Stretch, UiBundle, UiFinder, UiLabel, UiText, UiTransform},
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, GameData, SimpleState, SimpleTrans, StateData, Trans,
};

struct MouseRaycastSystem;

impl System<'_> for MouseRaycastSystem {
    fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MouseRaycastSystem")
                .with_query(<(&Camera, &Transform)>::query())
                .with_query(<(&SpriteRender, &Transform, &Named)>::query())
                .with_query(<&UiText>::query())
                .read_resource::<InputHandler>()
                .read_resource::<ActiveCamera>()
                .read_resource::<ScreenDimensions>()
                .read_resource::<AssetStorage<SpriteSheet>>()
                .read_resource::<AssetStorage<Sprites>>()
                //     Entities<'s>,
                //     WriteStorage<'s, UiText>,
                //     ReadExpect<'s, ScreenDimensions>,
                //     UiFinder<'s>,
                .build(
                    |_,
                     world,
                     (input, active_camera, screen_dimensions, sprite_sheets, sprites_storage),
                     (camera_query, sprite_query, ui_texts)| {
                        // Get the mouse position if its available
                        if let Some(mouse_position) = input.mouse_position() {
                            // Get the active camera if it is spawned and ready
                            if let Some((camera, camera_transform)) = active_camera
                                .entity
                                .and_then(|a| camera_query.get(world, a).ok())
                                .or_else(|| camera_query.iter(world).next())
                            {
                                // Project a ray from the camera to the 0z axis
                                let ray = camera.screen_ray(
                                    Point2::new(mouse_position.0, mouse_position.1),
                                    Vector2::new(
                                        screen_dimensions.width(),
                                        screen_dimensions.height(),
                                    ),
                                    camera_transform,
                                );
                                let distance = ray.intersect_plane(&Plane::with_z(0.0)).unwrap();
                                let mouse_world_position = ray.at_distance(distance);

                                if let Some(t) = UiFinder::find(world, "mouse_position")
                                    .and_then(|e| ui_texts.get_mut(world, e).ok())
                                {
                                    t.text = format!(
                                        "({:.0}, {:.0})",
                                        mouse_world_position.x, mouse_world_position.y
                                    );
                                }

                                // Find any sprites which the mouse is currently inside
                                let mut found_name = None;
                                for (sprite, transform, name) in sprite_query.iter(world) {
                                    let sprite_sheet =
                                        sprite_sheets.get(&sprite.sprite_sheet).unwrap();
                                    let sprites =
                                        sprites_storage.get(&sprite_sheet.sprites).unwrap();
                                    let sprite = &sprites.build_sprites()[sprite.sprite_number];
                                    let (min_x, max_x, min_y, max_y) = {
                                        // Sprites are centered on a coordinate, so we build out a bbox for the sprite coordinate
                                        // and dimensions
                                        // Notice we ignore z-axis for this example.
                                        (
                                            transform.translation().x - (sprite.width * 0.5),
                                            transform.translation().x + (sprite.width * 0.5),
                                            transform.translation().y - (sprite.height * 0.5),
                                            transform.translation().y + (sprite.height * 0.5),
                                        )
                                    };
                                    if mouse_world_position.x > min_x
                                        && mouse_world_position.x < max_x
                                        && mouse_world_position.y > min_y
                                        && mouse_world_position.y < max_y
                                    {
                                        found_name = Some(&name.0);
                                    }
                                }

                                if let Some(t) = UiFinder::find(world, "under_mouse")
                                    .and_then(|e| ui_texts.get_mut(world, e).ok())
                                {
                                    if let Some(name) = found_name {
                                        t.text = format!("{}", name);
                                    } else {
                                        t.text = "".to_string();
                                    }
                                }
                            }
                        }
                    },
                ),
        )
    }
}

/// The main state
#[derive(Default)]
struct Example {
    /// A progress tracker to check that assets are loaded
    pub progress_counter: Option<ProgressCounter>,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;
        // Crates new progress counter
        self.progress_counter = Some(Default::default());

        let sprite_sheet_handle = load_sprite_sheet(
            resources,
            "texture/cp437_20x20.png",
            "texture/cp437_20x20.ron",
            self.progress_counter.as_mut().unwrap(),
        );

        let _ = init_sprite(
            Vector3::new(0.0, 0.0, 0.0),
            "Sprite 1",
            15,
            world,
            &sprite_sheet_handle,
        );

        let _ = init_sprite(
            Vector3::new(100.0, 100.0, 0.0),
            "Sprite 2",
            15,
            world,
            &sprite_sheet_handle,
        );

        let _ = init_sprite(
            Vector3::new(-50.0, -50.0, 0.0),
            "Sprite 3",
            15,
            world,
            &sprite_sheet_handle,
        );

        let ui_transform = UiTransform {
            id: "background".into(),
            anchor: Anchor::Middle,
            stretch: Stretch::XY {
                x_margin: 0.,
                y_margin: 0.,
                keep_aspect_ratio: false,
            },
            width: 20.,
            height: 20.,
            ..Default::default()
        };

        let font = {
            resources
                .get::<DefaultLoader>()
                .unwrap()
                .load("font/square.ttf")
        };

        let text = UiText {
            font: Some(font),
            text: "N/A".into(),
            color: [1., 1., 1., 1.],
            font_size: 25.,
            ..Default::default()
        };

        init_camera(world, resources);
    }

    fn update(&mut self, _: &mut StateData<'_, GameData>) -> SimpleTrans {
        Trans::None
    }
}

// Initialize a sprite as a reference point at a fixed location
fn init_sprite(
    position: Vector3<f32>,
    name: &'static str,
    sprite_number: usize,
    world: &mut World,
    sprite_sheet: &Handle<SpriteSheet>,
) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation(position);

    let sprite = SpriteRender::new(sprite_sheet.clone(), sprite_number);
    world.push((transform, sprite, name))
}

fn init_camera(world: &mut World, resources: &mut Resources) {
    let (width, height) = {
        let dim = resources.get::<ScreenDimensions>().unwrap();
        (dim.width(), dim.height())
    };

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(1.0);

    world.push((camera_transform, Camera::standard_2d(width, height)))
}

fn load_sprite_sheet<P>(
    resources: &mut Resources,
    png_path: &str,
    ron_path: &str,
    progress: P,
) -> Handle<SpriteSheet>
where
    P: Progress,
{
    let loader = resources
        .get_mut::<DefaultLoader>()
        .expect("Missing loader");

    let texture = loader.load(png_path);
    let sprites = loader.load(ron_path);

    loader.load_from_data(
        SpriteSheet { texture, sprites },
        progress,
        &resources.get().expect("processing queue for SpriteSheet"),
    )
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("assets/");
    let display_config_path = app_root.join("config/display.ron");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(TransformBundle::new())?
        .add_bundle(InputBundle::new())?
        .add_bundle(UiBundle::new())?
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderUi::default())
                .with_plugin(RenderFlat2D::default()),
        )?
        .with(MouseRaycastSystem, "MouseRaycastSystem", &["input_system"]);

    let game = Application::build(assets_dir, Example::default())?.build(game_data)?;
    game.run();

    Ok(())
}

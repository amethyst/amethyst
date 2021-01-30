//! Demonstrates how to perform raycasts with the camera to project from mouse to world coordinates.

use amethyst::{
    assets::{
        AssetStorage, DefaultLoader, Handle, Loader, LoaderBundle, Progress, ProgressCounter,
    },
    core::{
        geometry::Plane,
        math::{Point2, Vector2, Vector3},
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
        RenderingBundle,
    },
    ui::{Anchor, RenderUi, UiBundle, UiText, UiTransform},
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, GameData, SimpleState, SimpleTrans, StateData, Trans,
};

struct MouseRaycastSystem;

impl System for MouseRaycastSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MouseRaycastSystem")
                .with_query(<(&Camera, &Transform)>::query())
                .with_query(<(&SpriteRender, &Transform, &Named)>::query())
                .with_query(<(&UiTransform, &mut UiText)>::query())
                .read_resource::<InputHandler>()
                .read_resource::<ActiveCamera>()
                .read_resource::<ScreenDimensions>()
                .read_resource::<AssetStorage<SpriteSheet>>()
                .read_resource::<AssetStorage<Sprites>>()
                .build(
                    |_,
                     world,
                     (input, active_camera, screen_dimensions, sprite_sheets, sprites_storage),
                     (camera_query, sprite_query, ui_texts)| {
                        // Get the mouse position if its available
                        if let Some(mouse_position) = input.mouse_position() {
                            // Get the active camera if it is spawned and ready
                            let (left, mut right) = world.split_for_query(camera_query);
                            if let Some((camera, camera_transform)) = active_camera
                                .entity
                                .and_then(|a| camera_query.get(&left, a).ok())
                                .or_else(|| camera_query.iter(&left).next())
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

                                for (transform, text) in ui_texts.iter_mut(&mut right) {
                                    if transform.id == "mouse_position" {
                                        text.text = format!(
                                            "({:.0}, {:.0})",
                                            mouse_world_position.x, mouse_world_position.y
                                        );
                                    }
                                }

                                // Find any sprites which the mouse is currently inside
                                let mut found_name = None;
                                let (left, mut right) = world.split_for_query(sprite_query);
                                for (sprite, transform, name) in sprite_query.iter(&left) {
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

                                for (transform, text) in ui_texts.iter_mut(&mut right) {
                                    if transform.id == "under_mouse" {
                                        if let Some(name) = found_name {
                                            text.text = format!("{}", name);
                                        } else {
                                            text.text = "".to_string();
                                        }
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

        let font = {
            resources
                .get::<DefaultLoader>()
                .unwrap()
                .load("font/square.ttf")
        };

        let text = UiText::new(
            Some(font.clone()),
            "Hippopotamus".into(),
            [1., 1., 1., 1.],
            25.,
            amethyst::ui::LineMode::Single,
            Anchor::Middle,
        );

        let transform = UiTransform {
            id: "mouse_position".into(),
            anchor: Anchor::TopLeft,
            local_x: 100.,
            local_y: -25.,
            width: 200.,
            height: 50.,
            opaque: false,
            ..Default::default()
        };

        world.push((text, transform));

        let text = UiText::new(
            Some(font),
            "Rhinoceros".into(),
            [1., 1., 1., 1.],
            25.,
            amethyst::ui::LineMode::Single,
            Anchor::Middle,
        );

        let transform = UiTransform {
            id: "under_mouse".into(),
            anchor: Anchor::TopLeft,
            local_x: 100.,
            local_y: -50.,
            width: 200.,
            height: 50.,
            opaque: false,
            ..Default::default()
        };

        world.push((text, transform));

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
    world.push((transform, sprite, Named(name.into())))
}

fn init_camera(world: &mut World, resources: &mut Resources) -> Entity {
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
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle::default())
        .add_bundle(InputBundle::default())
        .add_bundle(UiBundle::<u32>::default())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderUi::default())
                .with_plugin(RenderFlat2D::default()),
        )
        .add_system(MouseRaycastSystem);

    let game = Application::build(assets_dir, Example::default())?.build(game_data)?;
    game.run();

    Ok(())
}

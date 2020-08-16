//! Demonstrates how to perform raycasts with the camera to project from mouse to world coordinates.
//!
//!

use amethyst::{
    assets::{AssetStorage, Handle, Loader, Progress, ProgressCounter},
    core::{
        geometry::Plane,
        math::{Point2, Vector2, Vector3},
        transform::{Transform, TransformBundle},
        Named, WithNamed,
    },
    derive::SystemDesc,
    ecs::{
        prelude::Entity, Entities, Join, Read, ReadExpect, ReadStorage, System, SystemData,
        WriteStorage,
    },
    input::{InputBundle, InputHandler, StringBindings},
    prelude::{Builder, World, WorldExt},
    renderer::{
        camera::{ActiveCamera, Camera},
        plugins::{RenderFlat2D, RenderToWindow},
        sprite::{SpriteRender, SpriteSheet, SpriteSheetFormat},
        types::DefaultBackend,
        ImageFormat, RenderingBundle, Texture,
    },
    ui::{RenderUi, UiBundle, UiCreator, UiFinder, UiText},
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, GameData, GameDataBuilder, SimpleState, SimpleTrans, StateData, Trans,
};

#[derive(SystemDesc)]
struct MouseRaycastSystem;

impl<'s> System<'s> for MouseRaycastSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, SpriteRender>,
        ReadStorage<'s, Named>,
        WriteStorage<'s, UiText>,
        Read<'s, AssetStorage<SpriteSheet>>,
        ReadExpect<'s, ScreenDimensions>,
        Read<'s, ActiveCamera>,
        Read<'s, InputHandler<StringBindings>>,
        UiFinder<'s>,
    );

    fn run(
        &mut self,
        (
            entities,
            transforms,
            cameras,
            sprites,
            names,
            mut ui_texts,
            sprite_sheets,
            screen_dimensions,
            active_camera,
            input,
            ui_finder,
        ): Self::SystemData,
    ) {
        // Get the mouse position if its available
        if let Some(mouse_position) = input.mouse_position() {
            // Get the active camera if it is spawned and ready
            let mut camera_join = (&cameras, &transforms).join();
            if let Some((camera, camera_transform)) = active_camera
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next())
            {
                // Project a ray from the camera to the 0z axis
                let ray = camera.screen_ray(
                    Point2::new(mouse_position.0, mouse_position.1),
                    Vector2::new(screen_dimensions.width(), screen_dimensions.height()),
                    camera_transform,
                );
                let distance = ray.intersect_plane(&Plane::with_z(0.0)).unwrap();
                let mouse_world_position = ray.at_distance(distance);

                if let Some(t) = ui_finder
                    .find("mouse_position")
                    .and_then(|e| ui_texts.get_mut(e))
                {
                    t.text = format!(
                        "({:.0}, {:.0})",
                        mouse_world_position.x, mouse_world_position.y
                    );
                }

                // Find any sprites which the mouse is currently inside
                let mut found_name = None;
                for (sprite, transform, name) in (&sprites, &transforms, &names).join() {
                    let sprite_sheet = sprite_sheets.get(&sprite.sprite_sheet).unwrap();
                    let sprite = &sprite_sheet.sprites[sprite.sprite_number];
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
                        found_name = Some(&name.name);
                    }
                }

                if let Some(t) = ui_finder
                    .find("under_mouse")
                    .and_then(|e| ui_texts.get_mut(e))
                {
                    if let Some(name) = found_name {
                        t.text = format!("{}", name);
                    } else {
                        t.text = "".to_string();
                    }
                }
            }
        }
    }
}

/// The main state
#[derive(Default)]
struct Example {
    /// A progress tracker to check that assets are loaded
    pub progress_counter: Option<ProgressCounter>,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        // Crates new progress counter
        self.progress_counter = Some(Default::default());

        let sprite_sheet_handle = load_sprite_sheet(
            world,
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

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create(
                "ui/mouse_raycast.ron",
                self.progress_counter.as_mut().unwrap(),
            );
        });

        init_camera(world);
    }

    fn update(&mut self, _: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
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
    world
        .create_entity()
        .with(transform)
        .with(sprite)
        .named(name)
        .build()
}

fn init_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    //println!("Init camera with dimensions: {}x{}", width, height);

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(1.0);

    world
        .create_entity()
        .with(camera_transform)
        .with(Camera::standard_2d(width, height))
        .build();
}

fn load_sprite_sheet<P>(
    world: &mut World,
    png_path: &str,
    ron_path: &str,
    progress: P,
) -> Handle<SpriteSheet>
where
    P: Progress,
{
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
        progress,
        &sprite_sheet_store,
    )
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("examples/mouse_raycast/assets/");
    let display_config_path = app_root.join("examples/mouse_raycast/config/display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderUi::default())
                .with_plugin(RenderFlat2D::default()),
        )?
        .with(MouseRaycastSystem, "MouseRaycastSystem", &["input_system"]);

    let mut game = Application::new(assets_dir, Example::default(), game_data)?;
    game.run();

    Ok(())
}

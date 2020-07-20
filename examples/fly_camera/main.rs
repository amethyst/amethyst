//! Demonstrates how to use the fly camera

use amethyst::{
    assets::{AssetStorage, Loader},
    controls::{FlyControlBundle, HideCursor, FlyControl},
    core::{
        frame_limiter::FrameRateLimitStrategy,
        transform::{LocalToWorld, Rotation, Translation, TransformBundle},
    },
    input::{is_key_down, is_mouse_button_down, InputBundle, StringBindings},
    prelude::*,
    renderer::{
        camera::Camera,
        plugins::{RenderShaded3D, RenderToWindow, RenderDebugLines},
        mtl::{Material, MaterialDefaults},
        palette::{LinSrgba, Srgb},
        light::{Light, PointLight},
        rendy::{
            mesh::{Normal, Position, Tangent, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        shape::Shape,
        types::DefaultBackend,
        RenderingBundle, Mesh, Texture,
    },
    utils::application_root_dir,
    winit::{MouseButton, VirtualKeyCode},
    window::ScreenDimensions,
    Error,
};

//type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        let mat_defaults = resources.get::<MaterialDefaults>().unwrap().0.clone();
        let loader = resources.get::<Loader>().unwrap();
        let mesh_storage = resources.get::<AssetStorage<Mesh>>().unwrap();
        let tex_storage = resources.get::<AssetStorage<Texture>>().unwrap();
        let mtl_storage = resources.get::<AssetStorage<Material>>().unwrap();

        println!("Load mesh");
        let (mesh, albedo) = {
            let mesh = loader.load_from_data(
                Shape::Sphere(32, 32)
                    .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(None)
                    .into(),
                (),
                &mesh_storage,
            );

            let albedo = loader.load_from_data(
                load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 0.5)).into(),
                (),
                &tex_storage,
            );

            (mesh, albedo)
        };

        println!("Create spheres");
        let spheres = (0..25).map(|n| {
            let i = n / 5;
            let j = n % 5;

            let roughness = 1.0f32 * (i as f32 / 4.0f32);
            let metallic = 1.0f32 * (j as f32 / 4.0f32);

            let pos = Translation::new(2.0f32 * (i - 2) as f32, 2.0f32 * (j - 2) as f32, 0.0);

            let mtl = {
                let metallic_roughness = loader.load_from_data(
                    load_from_linear_rgba(LinSrgba::new(0.0, roughness, metallic, 0.0)).into(),
                    (),
                    &tex_storage,
                );

                loader.load_from_data(
                    Material {
                        albedo: albedo.clone(),
                        metallic_roughness,
                        ..mat_defaults.clone()
                    },
                    (),
                    &mtl_storage,
                )
            };

            (LocalToWorld::identity(), pos, mesh.clone(), mtl)
        });

        world.insert((), spheres);

        println!("Create lights");
        let light1: Light = PointLight {
            intensity: 6.0,
            color: Srgb::new(0.8, 0.0, 0.0),
            ..PointLight::default()
        }
        .into();

        let light1_translation = Translation::new(6.0, 6.0, -6.0);

        let light2: Light = PointLight {
            intensity: 5.0,
            color: Srgb::new(0.0, 0.3, 0.7),
            ..PointLight::default()
        }
        .into();

        let light2_translation = Translation::new(6.0, -6.0, -6.0);

        world.insert(
            (),
            vec![
                (LocalToWorld::identity(), light1, light1_translation),
                (LocalToWorld::identity(), light2, light2_translation),
            ],
        );

        println!("Put camera");

        let translation = Translation::new(0.0, 0.0, -12.0);
        let rotation = Rotation::from_euler_angles(0.0, 0.0, 0.0);

        let (width, height) = {
            let dim = resources.get::<ScreenDimensions>().unwrap();
            (dim.width(), dim.height())
        };

        world.insert(
            (),
            vec![(
                LocalToWorld::identity(),
                Camera::standard_3d(width, height),
                translation,
                rotation,
                FlyControl,
            )],
        );

    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { resources, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                let mut hide_cursor = resources.get_mut::<HideCursor>().unwrap();
                hide_cursor.hide = false;
            } else if is_mouse_button_down(&event, MouseButton::Left) {
                let mut hide_cursor = resources.get_mut::<HideCursor>().unwrap();
                hide_cursor.hide = true;
            }
        }
        Trans::None
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/fly_camera/config/display.ron");
    let assets_dir = app_root.join("examples/assets");
    let key_bindings_path = app_root.join("examples/fly_camera/config/input.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<StringBindings>::new().with_bindings_from_file(&key_bindings_path)?)
        .with_bundle(
            FlyControlBundle::<StringBindings>::new(
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            )
            .with_sensitivity(0.1, 0.1),
        )
        .with_bundle(TransformBundle)
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderShaded3D::default()),
        );

    let mut game = Application::build(assets_dir, ExampleState)?
        .with_frame_limit(FrameRateLimitStrategy::Sleep, 60)
        .build(game_data)?;

    game.run();
    Ok(())
}

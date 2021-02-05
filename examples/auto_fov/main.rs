use amethyst::{
    assets::{
        prefab::Prefab, AssetStorage, DefaultLoader, Handle, Loader, LoaderBundle, ProcessingQueue,
    },
    core::{
        transform::{Transform, TransformBundle},
        Time,
    },
    ecs::{CommandBuffer, Entity, IntoQuery, ParallelRunnable, System, SystemBuilder},
    input::{is_close_requested, is_key_down, InputBundle, VirtualKeyCode},
    prelude::{
        Application, DispatcherBuilder, GameData, SimpleState, SimpleTrans, StateData, StateEvent,
        Trans,
    },
    renderer::{
        light::{Light, PointLight},
        loaders::load_from_linear_rgba,
        palette::{LinSrgba, Srgb},
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::{
            hal::command::ClearColor,
            mesh::{Normal, Position, Tangent, TexCoord},
        },
        shape::Shape,
        types::{DefaultBackend, MeshData, TextureData},
        Camera, Material, MaterialDefaults, Mesh, RenderingBundle,
    },
    ui::{RenderUi, UiBundle, UiText, UiTransform},
    utils::{
        application_root_dir,
        auto_fov::{AutoFov, AutoFovSystem},
        tag::{Tag, TagFinder},
    },
    window::ScreenDimensions,
    Error,
};

fn main() -> Result<(), Error> {
    let config = amethyst::LoggerConfig {
        level_filter: amethyst::LogLevelFilter::Debug,
        module_levels: vec![
            (
                "amethyst_assets".to_string(),
                amethyst::LogLevelFilter::Trace,
            ),
            ("distill_daemon".to_string(), amethyst::LogLevelFilter::Warn),
            ("distill_loader".to_string(), amethyst::LogLevelFilter::Warn),
        ],
        ..Default::default()
    };

    amethyst::start_logger(config);

    let app_dir = application_root_dir()?;
    let assets_dir = app_dir.join("assets/");
    let display_config_path = app_dir.join("config/display.ron");

    let mut game_data = DispatcherBuilder::default();

    game_data
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_system(AutoFovSystem)
        .add_system(ShowFovSystem)
        .add_bundle(InputBundle::new())
        .add_bundle(UiBundle::<u32>::new())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderShaded3D::default())
                .with_plugin(RenderUi::default()),
        );

    let game = Application::build(assets_dir, Loading::new())?.build(game_data)?;
    game.run();

    Ok(())
}

#[derive(Clone, Default)]
struct ShowFov;

struct Loading {
    loading_ui: Option<Handle<Prefab>>,
    fov_ui: Option<Handle<Prefab>>,
}

impl Loading {
    fn new() -> Self {
        Loading {
            loading_ui: None,
            fov_ui: None,
        }
    }
}

impl SimpleState for Loading {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.resources.insert(TagFinder::<ShowFov>::default());
        let loader = data.resources.get::<DefaultLoader>().unwrap();
        let loading_ui: Handle<Prefab> = loader.load("ui/loading.prefab");
        self.loading_ui = Some(loading_ui.clone());
        data.world.push((loading_ui,));
        let fov_ui = loader.load("ui/fov.prefab");
        self.fov_ui = Some(fov_ui);

        // let prefab = loader.load("prefab/auto_fov.ron");
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let loader = data.resources.get::<AssetStorage<Prefab>>().unwrap();

        let time = data.resources.get::<Time>().unwrap();

        if time.frame_number() % 60 == 0 {
            let mut query = <(Entity,)>::query();
            let entities: Vec<Entity> = query.iter(data.world).map(|(ent,)| *ent).collect();
            for entity in entities {
                if let Some(entry) = data.world.entry(entity) {
                    log::info!("{:?}: {:?}", entity, entry.archetype());
                    if let Ok(pos) = entry.get_component::<UiTransform>() {
                        log::info!("{:?}", pos);
                    }
                }
            }
        }

        if loader.get(self.fov_ui.as_ref().unwrap()).is_some() {
            Trans::Switch(Box::new(Example {
                fov_ui: self.fov_ui.take(),
            }))
        } else {
            Trans::None
        }
    }
}

struct Example {
    fov_ui: Option<Handle<Prefab>>,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        data.world.push((self.fov_ui.as_ref().unwrap().clone(),));
        let mut buffer = CommandBuffer::new(data.world);
        let mut query = <(Entity, &UiTransform)>::query();
        for (entity, transform) in query.iter(data.world) {
            if transform.id == "loading" {
                buffer.remove(*entity)
            }
        }
        buffer.flush(data.world);

        let loader = data.resources.get::<DefaultLoader>().unwrap();
        let mesh_storage = data.resources.get::<ProcessingQueue<MeshData>>().unwrap();
        let tex_storage = data
            .resources
            .get::<ProcessingQueue<TextureData>>()
            .unwrap();
        let mtl_storage = data.resources.get::<ProcessingQueue<Material>>().unwrap();

        let mesh: Handle<Mesh> = loader.load_from_data(
            Shape::Sphere(64, 64)
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

        let mtl: Handle<Material> = {
            let mat_defaults = data.resources.get::<MaterialDefaults>().unwrap().0.clone();

            loader.load_from_data(
                Material {
                    albedo,
                    ..mat_defaults
                },
                (),
                &mtl_storage,
            )
        };

        data.world.push((Transform::default(), mesh, mtl));

        let light1: Light = PointLight {
            intensity: 6.0,
            color: Srgb::new(0.8, 0.0, 0.0),
            ..PointLight::default()
        }
        .into();

        let mut light1_transform = Transform::default();
        light1_transform.set_translation_xyz(6.0, 6.0, -6.0);

        let light2: Light = PointLight {
            intensity: 5.0,
            color: Srgb::new(0.0, 0.3, 0.7),
            ..PointLight::default()
        }
        .into();

        let mut light2_transform = Transform::default();
        light2_transform.set_translation_xyz(6.0, -6.0, -6.0);

        data.world
            .extend(vec![(light1, light1_transform), (light2, light2_transform)]);

        let (width, height) = {
            let dim = data.resources.get::<ScreenDimensions>().unwrap();
            (dim.width(), dim.height())
        };

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 0.0, -4.0);
        transform.prepend_rotation_y_axis(std::f32::consts::PI);

        data.world.push((
            Camera::standard_3d(width, height),
            transform,
            AutoFov::default(),
            Tag::<ShowFov>::default(),
        ));
    }

    fn handle_event(&mut self, _: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(ref event) = event {
            if is_close_requested(event) || is_key_down(event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

struct ShowFovSystem;

impl System for ShowFovSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("ShowFovSystem")
                .read_resource::<ScreenDimensions>()
                .read_component::<UiText>()
                .read_component::<Camera>()
                .read_component::<Tag<ShowFov>>()
                .with_query(<(&UiTransform, &mut UiText)>::query())
                .with_query(<(Entity, &Tag<ShowFov>)>::query())
                .build(|_, world, screen, (ui_query, tag_query)| {
                    for (transform, mut text) in ui_query.iter_mut(world) {
                        if transform.id == "screen_aspect" {
                            let screen_aspect = screen.aspect_ratio();
                            text.text = format!("Screen Aspect Ratio: {:.2}", screen_aspect);
                        }
                    }

                    let (mut left, mut right) = world.split_for_query(ui_query);

                    let (lefter, righter) = right.split_for_query(tag_query);

                    if let Some(entity) = tag_query.iter(&lefter).map(|(ent, _)| *ent).next() {
                        if let Ok((camera,)) = <(&Camera,)>::query().get(&righter, entity) {
                            let camera_aspect =
                                (camera.matrix[(1, 1)] / camera.matrix[(0, 0)]).abs();

                            for (transform, mut text) in ui_query.iter_mut(&mut left) {
                                if transform.id == "camera_aspect" {
                                    text.text =
                                        format!("Camera Aspect Ratio: {:.2}", camera_aspect);
                                }

                                if transform.id == "camera_fov" {
                                    let fovy = (-1.0 / camera.matrix[(1, 1)]).atan() * 2.0;

                                    text.text = format!(
                                        "Camera Fov: ({:.2}, {:.2})",
                                        fovy * camera_aspect,
                                        fovy
                                    );
                                }
                            }
                        }
                    }
                }),
        )
    }
}

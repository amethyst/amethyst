use amethyst::{
    assets::{
        AssetHandle, AssetStorage, Completion, DefaultLoader, Handle, LoadStatus, Loader,
        LoaderBundle, ProgressCounter,
    },
    core::transform::TransformBundle,
    ecs::{CommandBuffer, Entity, IntoQuery, ParallelRunnable, System, SystemBuilder},
    input::{is_close_requested, is_key_down, InputBundle, VirtualKeyCode},
    prelude::{
        Application, DispatcherBuilder, GameData, SimpleState, SimpleTrans, StateData, StateEvent,
        Trans,
    },
    renderer::{
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        Camera, RenderingBundle,
    },
    ui::{RenderUi, UiBundle, UiFinder, UiLabel, UiText, UiTransform},
    utils::{
        application_root_dir,
        auto_fov::{AutoFov, AutoFovSystem},
        tag::{Tag, TagFinder},
    },
    window::ScreenDimensions,
    Error,
};
use lazy_static::__Deref;
use log::{error, info};

const CLEAR_COLOR: ClearColor = ClearColor {
    float32: [0.0, 0.0, 0.0, 1.0],
};

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_dir = application_root_dir()?;
    let assets_dir = app_dir.join("assets/ui/");
    let display_config_path = app_dir.join("config/display.ron");

    let mut game_data = DispatcherBuilder::default();

    game_data
        .add_bundle(LoaderBundle)
        .add_system(Box::new(AutoFovSystem))
        .add_system(Box::new(ShowFovSystem))
        .add_bundle(TransformBundle::default())
        .add_bundle(InputBundle::new())
        .add_bundle(UiBundle::<u32>::new())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(CLEAR_COLOR),
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
    progress: ProgressCounter,
    loading_ui: Option<Handle<UiLabel>>,
}

impl Loading {
    fn new() -> Self {
        Loading {
            progress: ProgressCounter::new(),
            loading_ui: None,
        }
    }
}

impl SimpleState for Loading {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.resources.insert(TagFinder::<ShowFov>::default());
        let loader = data.resources.get::<DefaultLoader>().unwrap();
        self.loading_ui = Some(loader.load("ui/loading.ron"));
        // let fov_ui = loader.load("ui/fov.ron");
        // let prefab = loader.load("prefab/auto_fov.ron");
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let loader = data.resources.get::<AssetStorage<UiLabel>>().unwrap();

        if let Some(label) = self.loading_ui.as_ref().unwrap().asset(loader.deref()) {
            println!("Label Loaded");
            let label_storage = data.resources.get::<AssetStorage<UiLabel>>();

            match self.progress.complete() {
                Completion::Loading => Trans::None,
                Completion::Failed => {
                    error!("Failed to load the scene");
                    Trans::Quit
                }
                Completion::Complete => {
                    info!("Loading finished. Moving to the main state.");
                    Trans::Switch(Box::new(Example))
                }
            }
        } else {
            println!("ui not yet loaded");
            Trans::None
        }
    }
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let mut buffer = CommandBuffer::new(data.world);
        let mut query = <(Entity, &UiTransform)>::query();
        for (entity, transform) in query.iter(data.world) {
            if transform.id == "loading" {
                buffer.remove(*entity)
            }
        }
        buffer.flush(data.world);
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

impl System<'_> for ShowFovSystem {
    fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("ShowFovSystem")
                // type SystemData = (
                //     TagFinder<'a, ShowFov>,
                //     UiFinder<'a>,
                //     WriteStorage<'a, UiText>,
                //     ReadStorage<'a, Camera>,
                // );
                .read_resource::<ScreenDimensions>()
                .read_resource::<TagFinder<ShowFov>>()
                .read_component::<UiText>()
                .read_component::<Tag<ShowFov>>()
                .with_query(<(&UiTransform, &mut UiText)>::query())
                .build(|_, world, (screen, camera_finder), ui_query| {
                    for (transform, mut text) in ui_query.iter_mut(world) {
                        if transform.id == "screen_aspect" {
                            let screen_aspect = screen.aspect_ratio();
                            text.text = format!("Screen Aspect Ratio: {:.2}", screen_aspect);
                        }
                    }

                    let (mut left, mut right) = world.split_for_query(ui_query);
                    if let Some(entity) = camera_finder.find(&mut right) {
                        if let Ok(camera) = <&Camera>::query().get(&right, entity) {
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

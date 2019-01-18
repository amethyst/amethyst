use amethyst::{
    assets::{
        Completion, Handle, Prefab, PrefabData, PrefabError, PrefabLoader, PrefabLoaderSystem,
        ProgressCounter, RonFormat,
    },
    core::{Transform, TransformBundle},
    derive::PrefabData,
    ecs::{Entity, ReadExpect, ReadStorage, System, WriteStorage},
    input::{is_close_requested, is_key_down, InputBundle},
    prelude::{
        Application, Builder, GameData, GameDataBuilder, SimpleState, SimpleTrans, StateData,
        StateEvent, Trans,
    },
    renderer::{
        Camera, CameraPrefab, DrawShaded, GraphicsPrefab, LightPrefab, PosNormTex,
        ScreenDimensions, VirtualKeyCode,
    },
    ui::{UiBundle, UiCreator, UiFinder, UiText},
    utils::{
        auto_fov::{AutoFov, AutoFovSystem},
        tag::{Tag, TagFinder},
    },
};

use log::{error, info};
use serde::{Deserialize, Serialize};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_dir = amethyst::utils::application_dir("examples")?;
    let display_config = app_dir.join("auto_fov/resources/display.ron");
    let assets = app_dir.join("assets");

    let game_data = GameDataBuilder::new()
        .with(PrefabLoaderSystem::<ScenePrefab>::default(), "prefab", &[])
        .with(AutoFovSystem, "auto_fov", &["prefab"]) // This makes the system adjust the camera right after it has been loaded (in the same frame), preventing any flickering
        .with(ShowFovSystem, "show_fov", &["auto_fov"])
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_basic_renderer(display_config, DrawShaded::<PosNormTex>::new(), true)?;

    let mut game = Application::build(assets, Loading::new())?.build(game_data)?;
    game.run();

    Ok(())
}

#[derive(Default, Deserialize, PrefabData, Serialize)]
#[serde(default)]
struct ScenePrefab {
    graphics: Option<GraphicsPrefab<Vec<PosNormTex>>>,
    transform: Option<Transform>,
    light: Option<LightPrefab>,
    camera: Option<CameraPrefab>,
    auto_fov: Option<AutoFov>, // `AutoFov` implements `PrefabData` trait
    show_fov_tag: Option<Tag<ShowFov>>,
}

#[derive(Clone, Default)]
struct ShowFov;

struct Loading {
    progress: ProgressCounter,
    scene: Option<Handle<Prefab<ScenePrefab>>>,
}

impl Loading {
    fn new() -> Self {
        Loading {
            progress: ProgressCounter::new(),
            scene: None,
        }
    }
}

impl SimpleState for Loading {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/loading.ron", &mut self.progress);
            creator.create("ui/fov.ron", &mut self.progress);
        });

        let handle = data.world.exec(|loader: PrefabLoader<'_, ScenePrefab>| {
            loader.load("prefab/auto_fov.ron", RonFormat, (), &mut self.progress)
        });
        self.scene = Some(handle);
    }

    fn update(&mut self, _: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        match self.progress.complete() {
            Completion::Loading => Trans::None,
            Completion::Failed => {
                error!("Failed to load the scene");
                Trans::Quit
            }
            Completion::Complete => {
                info!("Loading finished. Moving to the main state.");
                Trans::Switch(Box::new(Example {
                    scene: self.scene.take().unwrap(),
                }))
            }
        }
    }
}

struct Example {
    scene: Handle<Prefab<ScenePrefab>>,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.create_entity().with(self.scene.clone()).build();
        data.world
            .exec(|finder: UiFinder| finder.find("loading"))
            .map_or_else(
                || error!("Unable to find Ui Text `loading`"),
                |e| {
                    data.world
                        .delete_entity(e)
                        .unwrap_or_else(|err| error!("{}", err))
                },
            );
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
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

impl<'a> System<'a> for ShowFovSystem {
    type SystemData = (
        TagFinder<'a, ShowFov>,
        UiFinder<'a>,
        WriteStorage<'a, UiText>,
        ReadStorage<'a, Camera>,
        ReadExpect<'a, ScreenDimensions>,
    );

    fn run(&mut self, (tag_finder, ui_finder, mut ui_texts, cameras, screen): Self::SystemData) {
        let screen_aspect = screen.aspect_ratio();
        ui_finder
            .find("screen_aspect")
            .and_then(|e| ui_texts.get_mut(e))
            .map(|t| {
                t.text = format!("Screen Aspect Ratio: {:.2}", screen_aspect);
            });

        if let Some(entity) = tag_finder.find() {
            if let Some(camera) = cameras.get(entity) {
                let fovy = get_fovy(camera);
                let camera_aspect = get_aspect(camera);
                ui_finder
                    .find("camera_aspect")
                    .and_then(|e| ui_texts.get_mut(e))
                    .map(|t| {
                        t.text = format!("Camera Aspect Ratio: {:.2}", camera_aspect);
                    });
                ui_finder
                    .find("camera_fov")
                    .and_then(|e| ui_texts.get_mut(e))
                    .map(|t| {
                        t.text = format!("Camera Fov: ({:.2}, {:.2})", fovy * camera_aspect, fovy);
                    });
            }
        }
    }
}

fn get_fovy(camera: &Camera) -> f32 {
    (1.0 / camera.proj[(1, 1)]).atan() * 2.0
}

fn get_aspect(camera: &Camera) -> f32 {
    camera.proj[(1, 1)] / camera.proj[(0, 0)]
}

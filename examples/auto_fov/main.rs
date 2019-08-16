use amethyst::{
    assets::{
        Completion, Handle, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystemDesc,
        ProgressCounter, RonFormat,
    },
    core::{SystemDesc, Transform, TransformBundle},
    derive::{PrefabData, SystemDesc},
    ecs::{Entity, ReadExpect, ReadStorage, System, SystemData, World, WorldExt, WriteStorage},
    input::{is_close_requested, is_key_down, InputBundle, StringBindings},
    prelude::{
        Application, Builder, GameData, GameDataBuilder, SimpleState, SimpleTrans, StateData,
        StateEvent, Trans,
    },
    renderer::{
        camera::{Camera, CameraPrefab},
        formats::GraphicsPrefab,
        light::LightPrefab,
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::mesh::{Normal, Position, Tangent, TexCoord},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle, UiCreator, UiFinder, UiText},
    utils::{
        auto_fov::{AutoFov, AutoFovSystem},
        tag::{Tag, TagFinder},
    },
    window::ScreenDimensions,
    winit::VirtualKeyCode,
    Error,
};
use log::{error, info};
use serde::{Deserialize, Serialize};

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_dir = amethyst::utils::application_dir("examples")?;
    let display_config_path = app_dir.join("auto_fov/config/display.ron");
    let assets_dir = app_dir.join("assets");

    let game_data = GameDataBuilder::new()
        .with_system_desc(
            PrefabLoaderSystemDesc::<ScenePrefab>::default(),
            "prefab",
            &[],
        )
        // This makes the system adjust the camera right after it has been loaded (in the same
        // frame), preventing any flickering
        .with(AutoFovSystem::new(), "auto_fov", &["prefab"])
        .with(ShowFovSystem::new(), "show_fov", &["auto_fov"])
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path).with_clear(CLEAR_COLOR),
                )
                .with_plugin(RenderShaded3D::default())
                .with_plugin(RenderUi::default()),
        )?;

    let mut game = Application::build(assets_dir, Loading::new())?.build(game_data)?;
    game.run();

    Ok(())
}

#[derive(Default, Deserialize, PrefabData, Serialize)]
#[serde(default)]
struct ScenePrefab {
    graphics: Option<GraphicsPrefab<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>>,
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
            loader.load("prefab/auto_fov.ron", RonFormat, &mut self.progress)
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

#[derive(SystemDesc)]
struct ShowFovSystem;

impl ShowFovSystem {
    pub fn new() -> Self {
        Self
    }
}

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
        if let Some(t) = ui_finder
            .find("screen_aspect")
            .and_then(|e| ui_texts.get_mut(e))
        {
            t.text = format!("Screen Aspect Ratio: {:.2}", screen_aspect);
        }

        if let Some(entity) = tag_finder.find() {
            if let Some(camera) = cameras.get(entity) {
                let fovy = get_fovy(camera);
                let camera_aspect = get_aspect(camera);
                if let Some(t) = ui_finder
                    .find("camera_aspect")
                    .and_then(|e| ui_texts.get_mut(e))
                {
                    t.text = format!("Camera Aspect Ratio: {:.2}", camera_aspect);
                }
                if let Some(t) = ui_finder
                    .find("camera_fov")
                    .and_then(|e| ui_texts.get_mut(e))
                {
                    t.text = format!("Camera Fov: ({:.2}, {:.2})", fovy * camera_aspect, fovy);
                }
            }
        }
    }
}

fn get_fovy(camera: &Camera) -> f32 {
    (-1.0 / camera.as_matrix()[(1, 1)]).atan() * 2.0
}

fn get_aspect(camera: &Camera) -> f32 {
    (camera.as_matrix()[(1, 1)] / camera.as_matrix()[(0, 0)]).abs()
}

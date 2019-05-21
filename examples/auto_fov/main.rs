use amethyst::{
    assets::{
        Completion, Handle, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystem, ProgressCounter,
        RonFormat,
    },
    core::{Transform, TransformBundle},
    derive::PrefabData,
    ecs::{Entity, ReadExpect, ReadStorage, Resources, System, WriteStorage},
    input::{is_close_requested, is_key_down, InputBundle, StringBindings},
    prelude::{
        Application, Builder, GameData, GameDataBuilder, SimpleState, SimpleTrans, StateData,
        StateEvent, Trans,
    },
    renderer::{
        camera::{Camera, CameraPrefab},
        formats::GraphicsPrefab,
        light::LightPrefab,
        pass::DrawShadedDesc,
        rendy::{
            factory::Factory,
            graph::{
                present::PresentNode,
                render::{RenderGroupDesc, SubpassBuilder},
                GraphBuilder,
            },
            hal::{
                command::{ClearDepthStencil, ClearValue},
                format::Format,
                image,
            },
            mesh::{Normal, Position, Tangent, TexCoord},
        },
        system::{GraphCreator, RenderingSystem},
        types::{Backend, DefaultBackend},
    },
    ui::{DrawUiDesc, UiBundle, UiCreator, UiFinder, UiText},
    utils::{
        auto_fov::{AutoFov, AutoFovSystem},
        tag::{Tag, TagFinder},
    },
    window::{ScreenDimensions, WindowBundle},
    winit::{VirtualKeyCode, Window},
    Error,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_dir = amethyst::utils::application_dir("examples")?;
    let display_config_path = app_dir.join("auto_fov/resources/display.ron");
    let assets = app_dir.join("assets");

    let game_data = GameDataBuilder::new()
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        .with(PrefabLoaderSystem::<ScenePrefab>::default(), "prefab", &[])
        .with(AutoFovSystem::default(), "auto_fov", &["prefab"]) // This makes the system adjust the camera right after it has been loaded (in the same frame), preventing any flickering
        .with(ShowFovSystem, "show_fov", &["auto_fov"])
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(UiBundle::<DefaultBackend, StringBindings>::new())?
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            ExampleGraph::default(),
        ));

    let mut game = Application::build(assets, Loading::new())?.build(game_data)?;
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
    (-1.0 / camera.as_matrix()[(1, 1)]).atan() * 2.0
}

fn get_aspect(camera: &Camera) -> f32 {
    (camera.as_matrix()[(1, 1)] / camera.as_matrix()[(0, 0)]).abs()
}

#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

impl<B: Backend> GraphCreator<B> for ExampleGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    fn builder(&mut self, factory: &mut Factory<B>, res: &Resources) -> GraphBuilder<B, Resources> {
        self.dirty = false;

        use amethyst::shred::SystemData;

        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);

        let surface = factory.create_surface(&window);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind =
            image::Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        let mut graph_builder = GraphBuilder::new();

        let color = graph_builder.create_image(
            window_kind,
            1,
            factory.get_surface_format(&surface),
            Some(ClearValue::Color(CLEAR_COLOR.into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D16Unorm,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawUiDesc::new().builder())
                .with_group(DrawShadedDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let present_builder = PresentNode::builder(factory, surface, color).with_dependency(pass);

        graph_builder.add_node(present_builder);

        graph_builder
    }
}

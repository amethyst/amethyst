//! Demonstrates how to use a custom game data structure

use crate::{
    example_system::ExampleSystem,
    game_data::{CustomGameData, CustomGameDataBuilder},
};
use amethyst::{
    assets::{
        Completion, Handle, Prefab, PrefabLoader, PrefabLoaderSystem, ProgressCounter, RonFormat,
    },
    config::Config,
    core::transform::TransformBundle,
    ecs::{
        prelude::{Component, Entity},
        storage::NullStorage,
    },
    input::{is_close_requested, is_key_down, InputBundle},
    prelude::*,
    renderer::{
        DisplayConfig, DrawShaded, Pipeline, PosNormTex, RenderBundle, Stage, VirtualKeyCode,
    },
    ui::{DrawUi, UiBundle, UiCreator, UiLoader, UiPrefab},
    utils::{application_root_dir, fps_counter::FPSCounterBundle, scene::BasicScenePrefab},
    Error,
};

mod example_system;
mod game_data;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

pub struct DemoState {
    light_angle: f32,
    light_color: [f32; 4],
    camera_angle: f32,
}

#[derive(Default)]
struct Loading {
    progress: ProgressCounter,
    scene: Option<Handle<Prefab<MyPrefabData>>>,
    load_ui: Option<Entity>,
    paused_ui: Option<Handle<UiPrefab>>,
}
struct Main {
    scene: Handle<Prefab<MyPrefabData>>,
    paused_ui: Handle<UiPrefab>,
}
struct Paused {
    ui: Entity,
}

#[derive(Default)]
struct Tag;

impl Component for Tag {
    type Storage = NullStorage<Self>;
}

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for Loading {
    fn on_start(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        self.scene = Some(data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/renderable.ron", RonFormat, (), &mut self.progress)
        }));

        self.load_ui = Some(data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/fps.ron", &mut self.progress);
            creator.create("ui/loading.ron", &mut self.progress)
        }));
        self.paused_ui = Some(
            data.world
                .exec(|loader: UiLoader<'_>| loader.load("ui/paused.ron", &mut self.progress)),
        );
        data.world.add_resource::<DemoState>(DemoState {
            light_angle: 0.0,
            light_color: [1.0; 4],
            camera_angle: 0.0,
        });
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, CustomGameData<'_, '_>>,
        event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        if let StateEvent::Window(event) = event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
        }
        Trans::None
    }

    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.data.update(&data.world, true);
        match self.progress.complete() {
            Completion::Failed => {
                eprintln!("Failed loading assets");
                Trans::Quit
            }
            Completion::Complete => {
                println!("Assets loaded, swapping state");
                if let Some(entity) = self.load_ui {
                    let _ = data.world.delete_entity(entity);
                }
                Trans::Switch(Box::new(Main {
                    scene: self.scene.as_ref().unwrap().clone(),
                    paused_ui: self.paused_ui.as_ref().unwrap().clone(),
                }))
            }
            Completion::Loading => Trans::None,
        }
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for Paused {
    fn handle_event(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
        event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::Space) {
                let _ = data.world.delete_entity(self.ui);
                Trans::Pop
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.data.update(&data.world, false);
        Trans::None
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for Main {
    fn on_start(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        data.world.create_entity().with(self.scene.clone()).build();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
        event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::Space) {
                Trans::Push(Box::new(Paused {
                    ui: data
                        .world
                        .create_entity()
                        .with(self.paused_ui.clone())
                        .build(),
                }))
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.data.update(&data.world, true);
        Trans::None
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let resources_directory = app_root.join("examples/assets");

    let display_config_path =
        app_root.join("examples/custom_game_data/resources/display_config.ron");

    let display_config = DisplayConfig::load(display_config_path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new())
            .with_pass(DrawUi::new()),
    );
    let game_data = CustomGameDataBuilder::default()
        .with_base(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_running::<ExampleSystem>(ExampleSystem::default(), "example_system", &[])
        .with_base_bundle(TransformBundle::new())?
        .with_base_bundle(UiBundle::<String, String>::new())?
        .with_base_bundle(FPSCounterBundle::default())?
        .with_base_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?
        .with_base_bundle(InputBundle::<String, String>::new())?;

    let mut game = Application::build(resources_directory, Loading::default())?.build(game_data)?;
    game.run();

    Ok(())
}

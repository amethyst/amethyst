//! Demonstrates how to use a custom game data structure

extern crate amethyst;
extern crate rayon;

use amethyst::{Application, Error, State, StateData, Trans};
use amethyst::assets::{Completion, HotReloadBundle, ProgressCounter};
use amethyst::config::Config;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::TransformBundle;
use amethyst::ecs::prelude::{Component, Entity, Join, World};
use amethyst::ecs::storage::NullStorage;
use amethyst::input::InputBundle;
use amethyst::renderer::{DisplayConfig, DrawShaded, Event, Pipeline, PosNormTex, RenderBundle,
                         Stage};
use amethyst::ui::{DrawUi, UiBundle};
use amethyst::utils::fps_counter::FPSCounterBundle;

use game_data::{CustomGameData, CustomGameDataBuilder};
use graphic::{add_graphics_to_world, load_assets, Assets};
use ui::{create_load_ui, create_paused_ui};
use input::{is_exit, is_pause};
use example_system::ExampleSystem;

mod game_data;
mod ui;
mod graphic;
mod input;
mod example_system;

pub struct DemoState {
    light_angle: f32,
    light_color: [f32; 4],
    camera_angle: f32,
    fps_display: Entity,
}

#[derive(Default)]
struct Loading {
    progress: Option<ProgressCounter>,
}
struct Main;
struct Paused;

#[derive(Default)]
struct Tag;

impl Component for Tag {
    type Storage = NullStorage<Self>;
}

fn delete_state_tagged(world: &mut World) {
    let entities = (&*world.entities(), &world.read_storage::<Tag>())
        .join()
        .map(|(e, _)| e)
        .collect::<Vec<_>>();
    if let Err(err) = world.delete_entities(&entities) {
        eprintln!("Failed deleting entities: {}", err);
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b>> for Loading {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        data.world.register::<Tag>();
        self.progress = Some(load_assets(data.world));
        let font = data.world.read_resource::<Assets>().font.clone();
        let fps_display = create_load_ui(data.world, font.clone());
        data.world.add_resource::<DemoState>(DemoState {
            light_angle: 0.0,
            light_color: [1.0; 4],
            camera_angle: 0.0,
            fps_display,
        });
    }

    fn handle_event(
        &mut self,
        _: StateData<CustomGameData>,
        event: Event,
    ) -> Trans<CustomGameData<'a, 'b>> {
        if is_exit(event) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<CustomGameData>) -> Trans<CustomGameData<'a, 'b>> {
        data.data.update(&data.world, true);
        match self.progress.as_ref().unwrap().complete() {
            Completion::Failed => {
                eprintln!("Failed loading assets");
                Trans::Quit
            }
            Completion::Complete => {
                println!("Assets loaded, swapping state");
                delete_state_tagged(data.world);
                Trans::Switch(Box::new(Main))
            }
            Completion::Loading => Trans::None,
        }
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b>> for Paused {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        let font = data.world.read_resource::<Assets>().font.clone();
        create_paused_ui(data.world, font);
    }

    fn handle_event(
        &mut self,
        data: StateData<CustomGameData>,
        event: Event,
    ) -> Trans<CustomGameData<'a, 'b>> {
        if is_exit(event.clone()) {
            Trans::Quit
        } else if is_pause(event) {
            delete_state_tagged(data.world);
            Trans::Pop
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<CustomGameData>) -> Trans<CustomGameData<'a, 'b>> {
        data.data.update(&data.world, false);
        Trans::None
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b>> for Main {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        add_graphics_to_world(data.world);
    }

    fn handle_event(
        &mut self,
        _: StateData<CustomGameData>,
        event: Event,
    ) -> Trans<CustomGameData<'a, 'b>> {
        if is_exit(event.clone()) {
            Trans::Quit
        } else if is_pause(event) {
            Trans::Push(Box::new(Paused))
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<CustomGameData>) -> Trans<CustomGameData<'a, 'b>> {
        data.data.update(&data.world, true);
        Trans::None
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Could not run the example!");
        eprintln!("{}", error);
        ::std::process::exit(1);
    }
}

/// Wrapper around the main, so we can return errors easily.
fn run() -> Result<(), Error> {
    // Add our meshes directory to the asset loader.
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/examples/renderable/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new())
            .with_pass(DrawUi::new()),
    );
    let game_data = CustomGameDataBuilder::default()
        .with_running::<ExampleSystem>(ExampleSystem, "example_system", &[])
        .with_base_bundle(TransformBundle::new())?
        .with_base_bundle(UiBundle::<String, String>::new())?
        .with_base_bundle(HotReloadBundle::default())?
        .with_base_bundle(FPSCounterBundle::default())?
        .with_base_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?
        .with_base_bundle(InputBundle::<String, String>::new())?;

    let mut game = Application::build(resources_directory, Loading::default())?
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 0)
        .build(game_data)?;
    game.run();

    Ok(())
}

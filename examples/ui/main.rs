//! Displays a shaded sphere to the user.

extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, Processor, RonFormat};
use amethyst::audio::output::init_output;
use amethyst::audio::Source;
use amethyst::core::transform::TransformBundle;
use amethyst::core::Time;
use amethyst::ecs::prelude::{Entity, System, Write};
use amethyst::input::{is_close_requested, is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::{DrawShaded, PosNormTex};
use amethyst::shrev::{EventChannel, ReaderId};
use amethyst::ui::{UiBundle, UiCreator, UiEvent, UiFinder, UiText};
use amethyst::utils::fps_counter::{FPSCounter, FPSCounterBundle};
use amethyst::utils::scene::BasicScenePrefab;
use amethyst::winit::{Event, VirtualKeyCode};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct Example {
    fps_display: Option<Entity>,
}

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        // Initialise the scene with an object, a light and a camera.
        let handle = world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, (), ())
        });
        world.create_entity().with(handle).build();
        init_output(&mut world.res);
        world.exec(|mut creator: UiCreator| {
            creator.create("ui/example.ron", ());
        });
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, state_data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        let StateData { world, data } = state_data;
        data.update(&world);
        if self.fps_display.is_none() {
            world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("fps") {
                    self.fps_display = Some(entity);
                }
            });
        }
        let mut ui_text = world.write_storage::<UiText>();
        if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
            if world.read_resource::<Time>().frame_number() % 20 == 0 {
                let fps = world.read_resource::<FPSCounter>().sampled_fps();
                fps_display.text = format!("FPS: {:.*}", 2, fps);
            }
        }

        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_config_path = format!(
        "{}/examples/ui/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with(Processor::<Source>::new(), "source_processor", &[])
        .with(UiEventHandlerSystem::new(), "ui_event_handler", &[])
        .with_bundle(FPSCounterBundle::default())?
        .with_bundle(InputBundle::<String, String>::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), true)?;
    let mut game = Application::new(resources, Example { fps_display: None }, game_data)?;
    game.run();
    Ok(())
}

/// This shows how to handle UI events.
pub struct UiEventHandlerSystem {
    reader_id: Option<ReaderId<UiEvent>>,
}

impl UiEventHandlerSystem {
    pub fn new() -> Self {
        UiEventHandlerSystem { reader_id: None }
    }
}

impl<'a> System<'a> for UiEventHandlerSystem {
    type SystemData = Write<'a, EventChannel<UiEvent>>;

    fn run(&mut self, mut events: Self::SystemData) {
        if self.reader_id.is_none() {
            self.reader_id = Some(events.register_reader());
        }
        for ev in events.read(self.reader_id.as_mut().unwrap()) {
            info!("You just interacted with a ui element: {:?}", ev);
        }
    }
}

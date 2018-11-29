//! Displays a shaded sphere to the user.

extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, Processor, RonFormat},
    audio::{output::init_output, Source},
    core::{transform::TransformBundle, Time},
    ecs::prelude::{Entity, Read, Resources, System, SystemData, WriteExpect},
    input::{is_close_requested, is_key_down, InputBundle},
    prelude::*,
    renderer::{DrawShaded, PosNormTex},
    shrev::{EventChannel, ReaderId},
    ui::{UiBundle, UiCreator, UiEvent, UiFinder, UiText},
    utils::{
        application_root_dir,
        fps_counter::{FPSCounter, FPSCounterBundle},
        scene::BasicScenePrefab,
    },
    winit::VirtualKeyCode,
};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct Example {
    fps_display: Option<Entity>,
}

impl<'a, 'b> SimpleState<'a, 'b> for Example {
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

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans<'a, 'b> {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => {
                info!(
                    "[HANDLE_EVENT] You just interacted with a ui element: {:?}",
                    ui_event
                );
                Trans::None
            }
        }
    }

    fn update(&mut self, state_data: &mut StateData<GameData>) -> SimpleTrans<'a, 'b> {
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

    let app_root = application_root_dir();

    let display_config_path = format!("{}/examples/ui/resources/display.ron", app_root);

    let resources = format!("{}/examples/assets", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with(Processor::<Source>::new(), "source_processor", &[])
        .with(UiEventHandlerSystem, "ui_event_handler", &[])
        .with_bundle(FPSCounterBundle::default())?
        .with_bundle(InputBundle::<String, String>::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), true)?;
    let mut game = Application::new(resources, Example { fps_display: None }, game_data)?;
    game.run();
    Ok(())
}

/// This shows how to handle UI events.
pub struct UiEventHandlerSystem;

/// A resource for `UiEventHandlerSystem`.  Automatically created and managed by
/// `UiEventHandlerSystem`.
pub struct UiEventHandlerSystemData {
    reader_id: ReaderId<UiEvent>,
}

impl<'a> System<'a> for UiEventHandlerSystem {
    type SystemData = (
        Read<'a, EventChannel<UiEvent>>,
        WriteExpect<'a, UiEventHandlerSystemData>,
    );

    fn run(&mut self, (events, mut data): Self::SystemData) {
        // Reader id was just initialized above if empty
        for ev in events.read(&mut data.reader_id) {
            info!("[SYSTEM] You just interacted with a ui element: {:?}", ev);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        let reader_id = res.fetch_mut::<EventChannel<UiEvent>>().register_reader();
        res.insert(UiEventHandlerSystemData { reader_id });
    }
}

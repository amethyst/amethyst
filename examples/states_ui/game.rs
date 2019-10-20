use crate::pause::PauseMenuState;
use crate::util::delete_hierarchy;
use amethyst::{
    audio::output::init_output,
    core::Time,
    ecs::prelude::{Entity, WorldExt},
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiFinder, UiText},
    utils::fps_counter::FpsCounter,
    winit::VirtualKeyCode,
};

/// Main 'Game' state. Actually, it is mostly similar to the ui/main.rs content-wise.
/// The main differences include the added 'paused' field in the state, which is toggled when
/// 'pausing'.

#[derive(Default)]
pub struct Game {
    // If the Game is paused or not
    paused: bool,
    // The UI root entity. Deleting this should remove the complete UI
    ui_root: Option<Entity>,
    // A reference to the FPS display, which we want to interact with
    fps_display: Option<Entity>,
    // A reference to the random text, which we want to modify during updates
    random_text: Option<Entity>,
}

impl SimpleState for Game {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { mut world, .. } = data;

        // needed for registering audio output.
        init_output(&mut world);

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/example.ron", ())));
    }

    fn on_pause(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        self.paused = true;
    }

    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        self.paused = false;
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(entity) = self.ui_root {
            delete_hierarchy(entity, data.world).expect("Failed to remove Game Screen");
        }
        self.ui_root = None;
        self.fps_display = None;
        self.random_text = None;
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Push] Pausing Game!");
                    Trans::Push(Box::new(PauseMenuState::default()))
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => {
                log::info!(
                    "[HANDLE_EVENT] You just interacted with a ui element: {:?}",
                    ui_event
                );
                Trans::None
            }
            StateEvent::Input(input) => {
                log::info!("Input Event detected: {:?}.", input);
                Trans::None
            }
        }
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = state_data;

        // this cannot happen in 'on_start', as the entity might not be fully
        // initialized/registered/created yet.
        if self.fps_display.is_none() {
            world.exec(|finder: UiFinder<'_>| {
                if let Some(entity) = finder.find("fps") {
                    self.fps_display = Some(entity);
                }
            });
        }

        if self.random_text.is_none() {
            world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("random_text") {
                    self.random_text = Some(entity);
                }
            });
        }

        // it is important that the 'paused' field is actually pausing your game.
        // Make sure to also pause your running systems.
        if !self.paused {
            let mut ui_text = world.write_storage::<UiText>();

            if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
                if world.read_resource::<Time>().frame_number() % 20 == 0 && !self.paused {
                    let fps = world.read_resource::<FpsCounter>().sampled_fps();
                    fps_display.text = format!("FPS: {:.*}", 2, fps);
                }
            }

            if let Some(random_text) = self.random_text.and_then(|entity| ui_text.get_mut(entity)) {
                if let Ok(value) = random_text.text.parse::<i32>() {
                    let mut new_value = value * 10;
                    if new_value > 100_000 {
                        new_value = 1;
                    }
                    random_text.text = new_value.to_string();
                } else {
                    random_text.text = String::from("1");
                }
            }
        }

        Trans::None
    }
}

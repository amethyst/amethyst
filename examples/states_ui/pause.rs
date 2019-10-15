use amethyst::{
    ecs::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    shrev::EventChannel,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
    TransEvent,
};

use crate::menu::MainMenu;

/// Adapted, originally from amethyst/evoli src/states/pause_menu.rs

#[derive(Default)]
pub struct PauseMenuState {
    // button entities are created in on_start() and destroyed in on_stop()
    resume_button: Option<Entity>,
    exit_to_main_menu_button: Option<Entity>,
    exit_button: Option<Entity>,
    root: Option<Entity>,
}

// ID's for buttons in the prefab. Required to identify them.
const RESUME_BUTTON_ID: &str = "resume";
const EXIT_TO_MAIN_MENU_BUTTON_ID: &str = "exit_to_main_menu";
const EXIT_BUTTON_ID: &str = "exit";

// load the pause_menu.ron prefab then instantiate it
// if the "resume" button is clicked, goto MainGameState
// if the "exit_to_main_menu" button is clicked, remove the pause and main game states and go to MenuState.
// if the "exit" button is clicked, quit the program.
impl<'a> SimpleState for PauseMenuState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        self.root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/pause_menu.ron", ())));
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(root) = self.root {
            if data.world.delete_entity(root).is_ok() {
                self.root = None;
            }
        }
        self.resume_button = None;
        self.exit_to_main_menu_button = None;
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Pop] Closing Pause Menu!");
                    Trans::Pop
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                if Some(target) == self.resume_button {
                    log::info!("Resuming Game!");
                    Trans::Pop
                } else if Some(target) == self.exit_to_main_menu_button {
                    let mut state_transition_event_channel = data
                        .world
                        .write_resource::<EventChannel<TransEvent<GameData, StateEvent>>>();

                    // this allows us to first 'Pop' this state, and then exchange whatever was
                    // below that with a new MainMenu state.
                    state_transition_event_channel.single_write(Box::new(|| Trans::Pop));
                    state_transition_event_channel
                        .single_write(Box::new(|| Trans::Switch(Box::new(MainMenu::default()))));

                    log::info!("[Trans::Pop] Closing Pause Menu!");
                    log::info!("[Trans::Switch] Switching to MainMenu!");

                    Trans::None // we could also not add the pop to the channel and Pop here
                                // but like this the execution order is guaranteed (in the next versions)
                } else if Some(target) == self.exit_button {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        // once deferred creation of the root ui entity finishes, look up buttons
        if self.resume_button.is_none()
            || self.exit_to_main_menu_button.is_none()
            || self.exit_button.is_none()
        {
            data.world.exec(|ui_finder: UiFinder<'_>| {
                self.resume_button = ui_finder.find(RESUME_BUTTON_ID);
                self.exit_to_main_menu_button = ui_finder.find(EXIT_TO_MAIN_MENU_BUTTON_ID);
                self.exit_button = ui_finder.find(EXIT_BUTTON_ID);
            });
        }
        Trans::None
    }
}

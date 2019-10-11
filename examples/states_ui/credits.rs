use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    ui::UiCreator,
    winit::{MouseButton, VirtualKeyCode},
};

use crate::menu::MainMenu;
use crate::util::delete_hierarchy;

// A simple 'Screen' State, only capable of loading/showing the prefab ui and registering simple
// UI interactions (pressing escape or clicking anywhere).

#[derive(Debug, Default)]
pub struct CreditsScreen {
    ui_handle: Option<Entity>,
}

impl SimpleState for CreditsScreen {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.ui_handle =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/credits.ron", ())));
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event)
                    || is_key_down(&event, VirtualKeyCode::Escape)
                    || is_mouse_button_down(&event, MouseButton::Left)
                {
                    Trans::Switch(Box::new(MainMenu::default()))
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(handler) = self.ui_handle {
            delete_hierarchy(handler, data.world).expect("Failed to remove CreditScreen");
        }
        self.ui_handle = None;
    }
}

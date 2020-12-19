use amethyst::prelude::*;

use crate::event::{GameEvent, MyExtendedStateEvent};

pub(crate) struct GameplayState {
    game_difficulty: i32,
}

impl Default for GameplayState {
    fn default() -> Self {
        GameplayState { game_difficulty: 0 }
    }
}

impl<'a, 'b> State<GameData, MyExtendedStateEvent> for GameplayState {
    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData>,
        event: MyExtendedStateEvent,
    ) -> Trans<GameData, MyExtendedStateEvent> {
        if let MyExtendedStateEvent::Game(GameEvent::IncreaseDifficulty) = event {
            self.game_difficulty += 1;
            println!(
                "Event received, game difficulty is now {}",
                self.game_difficulty
            );
        }
        Trans::None
    }

    fn update(&mut self, data: StateData<'_, GameData>) -> Trans<GameData, MyExtendedStateEvent> {
        data.data.update(data.world, data.resources);
        Trans::None
    }
}

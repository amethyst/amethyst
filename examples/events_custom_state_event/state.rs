use crate::event::{GameEvent, MyExtendedStateEvent};
use amethyst::prelude::*;

pub(crate) struct GameplayState {
    game_difficulty: i32,
}

impl Default for GameplayState {
    fn default() -> Self {
        GameplayState { game_difficulty: 0 }
    }
}

impl<'a, 'b> State<GameData<'a, 'b>, MyExtendedStateEvent> for GameplayState {
    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: MyExtendedStateEvent,
    ) -> Trans<GameData<'a, 'b>, MyExtendedStateEvent> {
        if let MyExtendedStateEvent::Game(GameEvent::IncreaseDifficulty) = event {
            self.game_difficulty += 1;
            println!(
                "Event received, game difficulty is now {}",
                self.game_difficulty
            );
        }
        Trans::None
    }

    fn update(
        &mut self,
        data: StateData<'_, GameData<'a, 'b>>,
    ) -> Trans<GameData<'a, 'b>, MyExtendedStateEvent> {
        data.data.update(&data.world);
        Trans::None
    }
}

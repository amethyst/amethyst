use crate::event::GameEvent;
use amethyst::{
    core::shrev::EventChannel,
    derive::SystemDesc,
    ecs::{System, SystemData, Write},
};

/// Signals the state when it's time to increase the game difficulty
#[derive(SystemDesc)]
pub(crate) struct IncreaseGameDifficultySystem;

impl<'a> System<'a> for IncreaseGameDifficultySystem {
    type SystemData = Write<'a, EventChannel<GameEvent>>;

    fn run(&mut self, mut my_event_channel: Self::SystemData) {
        my_event_channel.single_write(GameEvent::IncreaseDifficulty);
    }
}

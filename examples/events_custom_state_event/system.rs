use amethyst::{
    core::shrev::EventChannel,
    ecs::{systems::ParallelRunnable, *},
    Error,
};

use crate::event::GameEvent;

#[derive(Debug)]
pub(crate) struct MyBundle;

impl<'a, 'b> SystemBundle for MyBundle {
    fn load(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        let chan = EventChannel::<GameEvent>::default();
        resources.insert(chan);

        builder.add_system(build_difficulty_system());
        Ok(())
    }
}

/// Signals the state when it's time to increase the game difficulty
pub(crate) fn build_difficulty_system() -> impl ParallelRunnable {
    SystemBuilder::new("DifficultySystem")
        .write_resource::<EventChannel<GameEvent>>()
        .build(|_, _, my_event_channel, _| {
            my_event_channel.single_write(GameEvent::IncreaseDifficulty);
        })
}

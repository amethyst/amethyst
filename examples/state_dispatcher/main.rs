//! An example showing how to create a dispatcher inside of a State.

use amethyst::{
    ecs::{Dispatcher, DispatcherBuilder, WorldExt},
    prelude::*,
    shrev::EventChannel,
    utils::application_root_dir,
    Error,
};

use std::marker::PhantomData;

struct StateA;

impl SimpleState for StateA {
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        println!("StateA::update()");
        // Shows how to push a `Trans` through the event queue.
        // If you do use TransQueue, you will be forced to use the 'static lifetime on your states.
        data.world
            .write_resource::<EventChannel<TransEvent<GameData<'static, 'static>, StateEvent>>>()
            .single_write(Box::new(|| Trans::Push(Box::new(StateB::default()))));

        // You can also use normal Trans at the same time!
        // Those will be executed before the ones in the EventChannel
        // Trans::Push(Box::new(StateB::default()))

        Trans::None
    }
}

/// StateB isn't Send + Sync
struct StateB<'a> {
    dispatcher: Dispatcher<'static, 'static>,
    _phantom: &'a PhantomData<()>,
}

impl<'a> Default for StateB<'a> {
    fn default() -> Self {
        StateB {
            dispatcher: DispatcherBuilder::new().build(),
            _phantom: &PhantomData,
        }
    }
}

impl<'a> SimpleState for StateB<'a> {
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        println!("StateB::update()");
        self.dispatcher.dispatch(&data.world);
        Trans::Quit
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());
    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("examples/assets");
    let mut game = Application::build(assets_dir, StateA)?.build(GameDataBuilder::default())?;
    game.run();
    Ok(())
}

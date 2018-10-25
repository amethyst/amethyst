//! An example showing how to create a dispatcher inside of a State.

extern crate amethyst;

use amethyst::ecs::{Dispatcher, DispatcherBuilder};
use amethyst::prelude::*;
use amethyst::Error;

struct StateA;

impl SimpleState<'static, 'static> for StateA {
    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans<'static, 'static> {
        println!("StateA::update()");
        // Shows how to push a `Trans` through the event queue.
        // If you do use TransQueue, you will be forced to use the 'static lifetime on your states.
        data.world
            .write_resource::<TransQueue<GameData<'static, 'static>, StateEvent>>()
            .push_back(Box::new(|| Trans::Push(Box::new(StateB::default()))));
        Trans::None
    }
}

/// StateB isn't Send + Sync
struct StateB {
    dispatcher: Dispatcher<'static, 'static>,
}

impl Default for StateB {
    fn default() -> Self {
        StateB {
            dispatcher: DispatcherBuilder::new().build(),
        }
    }
}

impl SimpleState<'static, 'static> for StateB {
    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans<'static, 'static> {
        println!("StateB::update()");
        self.dispatcher.dispatch(&mut data.world.res);
        Trans::Quit
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());
    let mut game = Application::build("./", StateA)?.build(GameDataBuilder::default())?;
    game.run();
    Ok(())
}

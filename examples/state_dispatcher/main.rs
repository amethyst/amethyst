//! An example showing how to create a dispatcher inside of a State.

extern crate amethyst;

use amethyst::Error;
use amethyst::ecs::{Dispatcher, DispatcherBuilder};
use amethyst::prelude::*;

struct StateA;

impl SimpleState<'static, 'static> for StateA {
    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans<'static, 'static> {
        println!("StateA::update()");
        // Shows how to push a `Trans` through the event queue.
        data.world.write_resource::<TransQueue<GameData<'static, 'static>, StateEvent>>().push_back(Box::new(|| Trans::Push(Box::new(StateB::<'static, 'static>::default()))));
        Trans::None
    }
}

/// StateB isn't Send + Sync
struct StateB<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> Default for StateB<'a, 'b> {
    fn default() -> Self {
        StateB {
            dispatcher: DispatcherBuilder::new().build(),
        }
    }
}

impl<'a, 'b> SimpleState<'a, 'b> for StateB<'a, 'b> {
    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans<'a, 'b> {
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

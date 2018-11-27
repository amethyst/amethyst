//! An example showing how to create a dispatcher inside of a State.

#[macro_use]
extern crate amethyst;

use amethyst::{
    ecs::{Dispatcher, DispatcherBuilder},
    prelude::*,
    shrev::EventChannel,
    Error,
};

use std::marker::PhantomData;

#[derive(State, Clone, Debug)]
enum State {
    A,
    B,
}

struct StateA;

impl<E> StateCallback<State, E> for StateA {
    fn update(&mut self, world: &mut World) -> Trans<State> {
        println!("StateA::update()");
        // Shows how to push a `Trans` through the event queue.
        // If you do use TransQueue, you will be forced to use the 'static lifetime on your states.
        world
            .write_resource::<EventChannel<TransEvent<State>>>()
            .single_write(Box::new(|| Trans::Push(State::B)));

        // You can also use normal Trans at the same time!
        // Those will be executed before the ones in the EventChannel
        // Trans::Push(Box::new(StateB::default()))

        Trans::None
    }
}

/// StateB isn't Send + Sync
struct StateB<'a> {
    dispatcher: Dispatcher<'static, 'static>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Default for StateB<'a> {
    fn default() -> Self {
        StateB {
            dispatcher: DispatcherBuilder::new().build(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, S, E> StateCallback<S, E> for StateB<'a> {
    fn update(&mut self, world: &mut World) -> Trans<S> {
        println!("StateB::update()");
        self.dispatcher.dispatch(&mut world.res);
        Trans::Quit
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let mut game = Application::build("./")?
        .with_state(State::A, StateA)?
        .with_state(State::B, StateB::default())?
        .build(GameDataBuilder::default())?;

    game.run();
    Ok(())
}

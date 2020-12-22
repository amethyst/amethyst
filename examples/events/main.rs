extern crate amethyst;

use core::result::Result;

use amethyst::{
    core::{
        frame_limiter::FrameRateLimitStrategy,
        shrev::{EventChannel, ReaderId},
    },
    ecs::{DispatcherBuilder, World},
    prelude::*,
    utils::application_root_dir,
    Error,
};
use systems::ParallelRunnable;

#[derive(Debug)]
struct MyBundle;

impl<'a, 'b> SystemBundle for MyBundle {
    fn load(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        let mut chan = EventChannel::<MyEvent>::default();
        let reader = chan.register_reader();
        resources.insert(chan);

        builder.add_system(build_spamming_system());
        builder.add_system(build_receiving_system(reader));
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct AppEvent {
    data: i32,
}

#[derive(Debug)]
pub enum MyEvent {
    A,
    B,
    C,
}

struct GameplayState;

fn build_spamming_system() -> impl ParallelRunnable {
    SystemBuilder::new("SpamSystem")
        .write_resource::<EventChannel<MyEvent>>()
        .build(move |_, _, my_event_channel, _| {
            my_event_channel.single_write(MyEvent::A);
            println!("Sending A");
            my_event_channel.single_write(MyEvent::B);
            println!("Sending B");
            my_event_channel.single_write(MyEvent::C);
            println!("Sending C");
        })
}

fn build_receiving_system(mut reader: ReaderId<MyEvent>) -> impl ParallelRunnable {
    SystemBuilder::new("SpamSystem")
        .read_resource::<EventChannel<MyEvent>>()
        .build(move |_, _, my_event_channel, _| {
            for event in my_event_channel.read(&mut reader) {
                println!("Received an event: {:?}", event);
            }
        })
}

impl SimpleState for GameplayState {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("examples/events/assets");

    let mut game_data = DispatcherBuilder::default();
    game_data.add_bundle(MyBundle);

    let game = Application::build(assets_dir, GameplayState)?
        .with_frame_limit(FrameRateLimitStrategy::Sleep, 1)
        .build(game_data)?;

    game.run();
    Ok(())
}

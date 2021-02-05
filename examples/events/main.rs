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
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        let mut chan = EventChannel::<MyEvent>::default();
        let reader = chan.register_reader();
        resources.insert(chan);

        builder.add_system(SpammingSystem);
        builder.add_system(SpamReceiverSystem { reader });
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

struct SpammingSystem;

impl System for SpammingSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("SpamSystem")
                .write_resource::<EventChannel<MyEvent>>()
                .build(move |_, _, my_event_channel, _| {
                    my_event_channel.single_write(MyEvent::A);
                    println!("Sending A");
                    my_event_channel.single_write(MyEvent::B);
                    println!("Sending B");
                    my_event_channel.single_write(MyEvent::C);
                    println!("Sending C");
                }),
        )
    }
}

struct SpamReceiverSystem {
    reader: ReaderId<MyEvent>,
}

impl System for SpamReceiverSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("SpamSystem")
                .read_resource::<EventChannel<MyEvent>>()
                .build(move |_, _, my_event_channel, _| {
                    for event in my_event_channel.read(&mut self.reader) {
                        println!("Received an event: {:?}", event);
                    }
                }),
        )
    }
}

impl SimpleState for GameplayState {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("assets");

    let mut game_data = DispatcherBuilder::default();
    game_data.add_bundle(MyBundle);

    let game = Application::build(assets_dir, GameplayState)?
        .with_frame_limit(FrameRateLimitStrategy::Sleep, 1)
        .build(game_data)?;

    game.run();
    Ok(())
}

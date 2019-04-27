extern crate amethyst;

use amethyst::core::{
    bundle::SystemBundle,
    frame_limiter::FrameRateLimitStrategy,
    shrev::{EventChannel, ReaderId},
};
use amethyst::{
    ecs::{DispatcherBuilder, Read, Resources, System, SystemData, World, Write},
    prelude::*,
};

use amethyst::Error;
use core::result::Result;

#[derive(Debug)]
struct MyBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MyBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(SpammingSystem, "spamming_system", &[]);
        builder.add(
            ReceivingSystem {
                reader: Option::None,
            },
            "receiving_system",
            &[],
        );
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

impl<'a> System<'a> for SpammingSystem {
    type SystemData = Write<'a, EventChannel<MyEvent>>;

    fn run(&mut self, mut my_event_channel: Self::SystemData) {
        my_event_channel.single_write(MyEvent::A);
        println!("Sending A");
        my_event_channel.single_write(MyEvent::B);
        println!("Sending B");
        my_event_channel.single_write(MyEvent::C);
        println!("Sending C");
    }
}

struct ReceivingSystem {
    reader: Option<ReaderId<MyEvent>>,
}

impl<'a> System<'a> for ReceivingSystem {
    type SystemData = Read<'a, EventChannel<MyEvent>>;

    fn run(&mut self, my_event_channel: Self::SystemData) {
        for event in my_event_channel.read(self.reader.as_mut().unwrap()) {
            println!("Received an event: {:?}", event);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(res.fetch_mut::<EventChannel<MyEvent>>().register_reader());
    }
}

impl SimpleState for GameplayState {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let mut world = World::new();
    world.add_resource(EventChannel::<MyEvent>::new());

    let game_data = GameDataBuilder::default().with_bundle(MyBundle)?;

    let mut game = Application::build("./", GameplayState)?
        .with_frame_limit(FrameRateLimitStrategy::Sleep, 1)
        .build(game_data)?;

    game.run();
    Ok(())
}

extern crate amethyst;

use amethyst::{
    core::{
        bundle::SystemBundle,
        frame_limiter::FrameRateLimitStrategy,
        shrev::{EventChannel, ReaderId},
        SystemDesc,
    },
    derive::SystemDesc,
    ecs::{DispatcherBuilder, Read, System, SystemData, World, Write},
    prelude::*,
    utils::application_root_dir,
};

use amethyst::Error;
use core::result::Result;

#[derive(Debug)]
struct MyBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MyBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(SpammingSystem, "spamming_system", &[]);
        builder.add(
            ReceivingSystemDesc::default().build(world),
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

#[derive(SystemDesc)]
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

#[derive(SystemDesc)]
#[system_desc(name(ReceivingSystemDesc))]
struct ReceivingSystem {
    #[system_desc(event_channel_reader)]
    reader: ReaderId<MyEvent>,
}

impl ReceivingSystem {
    pub fn new(reader: ReaderId<MyEvent>) -> Self {
        ReceivingSystem { reader }
    }
}

impl<'a> System<'a> for ReceivingSystem {
    type SystemData = Read<'a, EventChannel<MyEvent>>;

    fn run(&mut self, my_event_channel: Self::SystemData) {
        for event in my_event_channel.read(&mut self.reader) {
            println!("Received an event: {:?}", event);
        }
    }
}

impl SimpleState for GameplayState {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("./");

    let game_data = GameDataBuilder::default().with_bundle(MyBundle)?;

    let mut game = Application::build(assets_dir, GameplayState)?
        .with_frame_limit(FrameRateLimitStrategy::Sleep, 1)
        .build(game_data)?;

    game.run();
    Ok(())
}

extern crate amethyst;

use amethyst::{
    core::{
        bundle::SystemBundle,
        frame_limiter::FrameRateLimitStrategy,
        shrev::{EventChannel, ReaderId},
    },
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
        builder.add(ReceivingSystem::new(world), "receiving_system", &[]);
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
    reader: ReaderId<MyEvent>,
}

impl ReceivingSystem {
    pub fn new(mut world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(&mut world);
        let reader = world.fetch_mut::<EventChannel<MyEvent>>().register_reader();
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
    let mut world = World::with_application_resources::<GameData<'_, '_>, _>(assets_dir)?;
    world.insert(EventChannel::<MyEvent>::new());

    let game_data = GameDataBuilder::default().with_bundle(&mut world, MyBundle)?;

    let mut game = Application::build(GameplayState, world)?
        .with_frame_limit(FrameRateLimitStrategy::Sleep, 1)
        .build(game_data)?;

    game.run();
    Ok(())
}

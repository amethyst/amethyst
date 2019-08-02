use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, SystemDesc},
    derive::SystemDesc,
    ecs::{Component, Entities, Join, System, SystemData, VecStorage, World, WriteStorage},
    network::*,
    prelude::*,
    shrev::ReaderId,
    utils::application_root_dir,
    Result,
};

use log::info;

use std::time::Duration;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("./");

    let game_data = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<String>::new(
            "127.0.0.1:3455".parse().unwrap(),
        ))?
        .with(SpamReceiveSystem::new(), "rcv", &[]);
    let mut game = Application::build(assets_dir, State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            1,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}

/// Default empty state
pub struct State1;
impl SimpleState for State1 {
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {}
}

/// Component to store client's event subscription
struct SpamReader(ReaderId<NetEvent<String>>);

impl Component for SpamReader {
    type Storage = VecStorage<Self>;
}

/// A simple system that receives a ton of network events.
#[derive(SystemDesc)]
struct SpamReceiveSystem {}

impl SpamReceiveSystem {
    pub fn new() -> Self {
        SpamReceiveSystem {}
    }
}

impl<'a> System<'a> for SpamReceiveSystem {
    type SystemData = (
        WriteStorage<'a, NetConnection<String>>,
        WriteStorage<'a, SpamReader>,
        Entities<'a>,
    );
    fn run(&mut self, (mut connections, mut readers, entities): Self::SystemData) {
        let mut count = 0;
        let mut connection_count = 0;

        for (e, connection) in (&entities, &mut connections).join() {
            let reader = readers
                .entry(e)
                .expect("Cannot get reader")
                .or_insert_with(|| SpamReader(connection.register_reader()));

            let mut client_disconnected = false;

            for ev in connection.received_events(&mut reader.0) {
                count += 1;
                match ev {
                    NetEvent::Packet(packet) => info!("{}", packet.content()),
                    NetEvent::Connected(addr) => info!("New Client Connection: {}", addr),
                    NetEvent::Disconnected(_addr) => {
                        client_disconnected = true;
                    }
                    _ => {}
                }
            }

            if client_disconnected {
                println!("Client Disconnects");
                entities
                    .delete(e)
                    .expect("Cannot delete connection from world!");
            }

            connection_count += 1;
        }
        println!(
            "Received {} messages this frame connections: {}",
            count, connection_count
        );
    }
}

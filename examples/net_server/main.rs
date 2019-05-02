use amethyst::{
    core::frame_limiter::FrameRateLimitStrategy,
    ecs::{Entities, Join, System, WriteStorage},
    network::*,
    prelude::*,
    shrev::ReaderId,
    Result,
};

use log::info;

use std::time::Duration;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let game_data = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<String>::new(
            "127.0.0.1:3455".parse().unwrap(),
        ))?
        .with(SpamReceiveSystem::new(), "rcv", &[]);
    let mut game = Application::build("./", State1)?
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

/// A simple system that receives a ton of network events.
struct SpamReceiveSystem {
    pub reader: Option<ReaderId<NetEvent<String>>>,
}

impl SpamReceiveSystem {
    pub fn new() -> Self {
        SpamReceiveSystem { reader: None }
    }
}

impl<'a> System<'a> for SpamReceiveSystem {
    type SystemData = (WriteStorage<'a, NetConnection<String>>, Entities<'a>);
    fn run(&mut self, (mut connections, entities): Self::SystemData) {
        let mut count = 0;
        let mut connection_count = 0;

        for (e, connection) in (&entities, &mut connections).join() {
            if self.reader.is_none() {
                self.reader = Some(connection.receive_buffer.register_reader());
            }

            let mut client_disconnected = false;

            for ev in connection
                .receive_buffer
                .read(self.reader.as_mut().unwrap())
            {
                count += 1;
                match ev {
                    NetEvent::Packet(packet) => info!("{}", packet.content()),
                    NetEvent::Connected(addr) => println!("New Client Connection: {}", addr),
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

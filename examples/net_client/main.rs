use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, Time},
    ecs::{Join, Read, System, WriteStorage},
    network::*,
    prelude::*,
    Result,
};
use log::info;
use std::time::Duration;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let game_data = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<String>::new(
            "127.0.0.1:3457".parse().unwrap(),
            vec![],
        ))?
        .with(SpamSystem::new(), "spam", &[]);
    let mut game = Application::build("./", State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}
/// Default empty state
pub struct State1;
impl SimpleState for State1 {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world
            .create_entity()
            .with(NetConnection::<String>::new(
                "127.0.0.1:3455".parse().unwrap(),
            ))
            .build();
    }
}

/// A simple system that sends a ton of messages to all connections.
/// In this case, only the server is connected.
struct SpamSystem;

impl SpamSystem {
    pub fn new() -> Self {
        SpamSystem {}
    }
}

impl<'a> System<'a> for SpamSystem {
    type SystemData = (WriteStorage<'a, NetConnection<String>>, Read<'a, Time>);
    fn run(&mut self, (mut connections, time): Self::SystemData) {
        for conn in (&mut connections).join() {
            info!("Sending 10k messages.");
            for i in 0..10000 {
                let packet = NetEvent::Packet(NetPacket::unreliable(format!(
                    "CL: frame:{},abs_time:{},c:{}",
                    time.frame_number(),
                    time.absolute_time_seconds(),
                    i
                )));

                conn.send_buffer.single_write(packet);
            }
        }
    }
}

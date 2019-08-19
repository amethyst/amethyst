use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, SystemDesc, Time},
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, World, WriteStorage},
    network::*,
    prelude::*,
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
            "127.0.0.1:3457".parse().unwrap(),
        ))?
        .with(SpamSystem::new(), "spam", &[]);
    let mut game = Application::build(assets_dir, State1)?
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
#[derive(SystemDesc)]
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
            for i in 0..500 {
                let packet = NetEvent::Packet(NetPacket::unreliable(format!(
                    "CL: frame:{},abs_time:{},c:{}",
                    time.frame_number(),
                    time.absolute_time_seconds(),
                    i
                )));

                conn.queue(packet);
            }
        }
    }
}

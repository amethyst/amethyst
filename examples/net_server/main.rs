extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::{
    core::frame_limiter::FrameRateLimitStrategy,
    ecs::{Join, Resources, System, SystemData, WriteExpect, WriteStorage},
    network::*,
    prelude::*,
    shrev::{EventChannel, ReaderId},
    Result,
};
use std::time::Duration;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());
    let game_data = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<()>::new(
            "127.0.0.1:3456".parse().unwrap(),
            vec![Box::new(FilterConnected::<()>::new())],
        ))?.with(SpamReceiveSystem, "rcv", &[]);
    let mut game = Application::build("./", State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            1,
        ).build(game_data)?;
    game.run();
    Ok(())
}

/// Default empty state
pub struct State1;
impl<'a, 'b> SimpleState<'a, 'b> for State1 {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world
            .create_entity()
            .with(NetConnection::<()>::new("127.0.0.1:3455".parse().unwrap()))
            .build();
    }
}

/// A simple system that receives a ton of network events.
struct SpamReceiveSystem;

struct SpamReceiveSystemData {
    pub reader: ReaderId<NetEvent<()>>,
}

impl<'a> System<'a> for SpamReceiveSystem {
    type SystemData = (
        WriteStorage<'a, NetConnection<()>>,
        WriteExpect<'a, SpamReceiveSystemData>,
    );
    fn run(&mut self, (mut connections, mut data): Self::SystemData) {
        let mut count = 0;
        for (mut conn,) in (&mut connections,).join() {
            for ev in conn.receive_buffer.read(&mut data.reader) {
                count += 1;
                match ev {
                    &NetEvent::TextMessage { ref msg } => info!("{}", msg),
                    _ => {}
                }
            }
        }
        info!("Received {} messages this frame", count);
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        let reader = res
            .fetch_mut::<EventChannel<NetEvent<()>>>()
            .register_reader();
        res.insert(SpamReceiveSystemData { reader });
    }
}

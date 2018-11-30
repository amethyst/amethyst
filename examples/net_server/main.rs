extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::{
    core::frame_limiter::FrameRateLimitStrategy,
    ecs::{Join, System, WriteStorage},
    network::*,
    prelude::*,
    shrev::ReaderId,
    Result,
};
use std::time::Duration;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());
    let game_data = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<()>::new(
            "127.0.0.1:3456".parse().unwrap(),
            vec![Box::new(FilterConnected::<()>::new())],
        ))?.with(SpamReceiveSystem::new(), "rcv", &[]);

    let mut game = Application::build("./")?
        .with_state((), State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            1,
        ).build(game_data)?;

    game.run();
    Ok(())
}

/// Default empty state
pub struct State1;
impl<S, E> StateCallback<S, E> for State1 {
    fn on_start(&mut self, world: &mut World) {
        world
            .create_entity()
            .with(NetConnection::<()>::new("127.0.0.1:3455".parse().unwrap()))
            .build();
    }
}

/// A simple system that receives a ton of network events.
struct SpamReceiveSystem {
    pub reader: Option<ReaderId<NetEvent<()>>>,
}

impl SpamReceiveSystem {
    pub fn new() -> Self {
        SpamReceiveSystem { reader: None }
    }
}

impl<'a> System<'a> for SpamReceiveSystem {
    type SystemData = (WriteStorage<'a, NetConnection<()>>,);
    fn run(&mut self, (mut connections,): Self::SystemData) {
        let mut count = 0;
        for (mut conn,) in (&mut connections,).join() {
            if self.reader.is_none() {
                self.reader = Some(conn.receive_buffer.register_reader());
            }
            for ev in conn.receive_buffer.read(self.reader.as_mut().unwrap()) {
                count += 1;
                match ev {
                    &NetEvent::TextMessage { ref msg } => info!("{}", msg),
                    _ => {}
                }
            }
        }
        info!("Received {} messages this frame", count);
    }
}

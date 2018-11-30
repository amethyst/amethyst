extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, Time},
    ecs::{Join, Read, System, WriteStorage},
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
            "127.0.0.1:3455".parse().unwrap(),
            vec![],
        ))?.with(SpamSystem::new(), "spam", &[])
        .with(ReaderSystem::new(), "reader", &[]);

    let mut game = Application::build("./")?
        .with_state((), State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
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
            .with(NetConnection::<()>::new("127.0.0.1:3456".parse().unwrap()))
            .build();
    }
}

/// A simple system that sends a ton of messages to all connections.
/// In this case, only the server is connected.
struct SpamSystem {}

impl SpamSystem {
    pub fn new() -> Self {
        SpamSystem {}
    }
}

impl<'a> System<'a> for SpamSystem {
    type SystemData = (WriteStorage<'a, NetConnection<()>>, Read<'a, Time>);
    fn run(&mut self, (mut connections, time): Self::SystemData) {
        for mut conn in (&mut connections).join() {
            info!("Sending 10k messages.");
            for i in 0..10000 {
                let ev = NetEvent::TextMessage {
                    msg: format!(
                        "CL: frame:{},abs_time:{},c:{}",
                        time.frame_number(),
                        time.absolute_time_seconds(),
                        i
                    ),
                };

                conn.send_buffer.single_write(ev);
            }
        }
    }
}

/// A simple system reading received events.
/// Used to see events sent by the net_echo_server example.
struct ReaderSystem {
    pub reader: Option<ReaderId<NetEvent<()>>>,
}

impl ReaderSystem {
    pub fn new() -> Self {
        ReaderSystem { reader: None }
    }
}

impl<'a> System<'a> for ReaderSystem {
    type SystemData = (WriteStorage<'a, NetConnection<()>>,);
    fn run(&mut self, (mut connections,): Self::SystemData) {
        if let Some((conn,)) = (&mut connections,).join().next() {
            if self.reader.is_none() {
                self.reader = Some(conn.receive_buffer.register_reader());
            }
            for ev in conn.receive_buffer.read(self.reader.as_mut().unwrap()) {
                match ev {
                    NetEvent::TextMessage { ref msg } => info!("Received: {}", msg),
                    _ => {}
                }
            }
        }
    }
}

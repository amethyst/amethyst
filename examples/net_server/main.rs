extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::ecs::{Join, System, WriteStorage};
use amethyst::network::*;
use amethyst::prelude::*;
use amethyst::shrev::ReaderId;
use amethyst::Result;
use std::time::Duration;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());
    let game_data = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<()>::new_server(
            "127.0.0.1",
            Some(3456 as u16),
            vec![Box::new(FilterConnected::<()>::new())],
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
impl<'a, 'b> State<GameData<'a, 'b>> for State1 {
    fn on_start(&mut self, mut data: StateData<GameData>) {
        data.world
            .create_entity()
            .with(NetConnection::<()>::new("127.0.0.1:3455".parse().unwrap()))
            .build();
    }

    fn update(&mut self, mut data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&mut data.world);
        Trans::None
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
                    &NetEvent::TextMessage { ref msg } => {} //println!("{}", msg),
                    _ => {}
                }
            }
        }
        println!("Received {} messages this frame", count);
    }
}

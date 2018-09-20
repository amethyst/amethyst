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
        .with(EchoSystem::new(), "echo", &[]);
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
    fn on_start(&mut self, data: StateData<GameData>) {
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

/// The echo system sends any received event to all connected clients.
struct EchoSystem {
    pub reader: Option<ReaderId<NetEvent<()>>>,
}

impl EchoSystem {
    pub fn new() -> Self {
        EchoSystem { reader: None }
    }
}

impl<'a> System<'a> for EchoSystem {
    type SystemData = (WriteStorage<'a, NetConnection<()>>,);
    fn run(&mut self, (mut connections,): Self::SystemData) {
        if let Some((conn,)) = (&mut connections,).join().next() {
            if self.reader.is_none() {
                self.reader = Some(conn.receive_buffer.register_reader());
            }
            for ev in conn.receive_buffer.read(self.reader.as_mut().unwrap()) {
                conn.send_buffer.single_write(ev.clone());
            }
        }
    }
}

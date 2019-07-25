use amethyst::{
    core::frame_limiter::FrameRateLimitStrategy,
    ecs::{Join, System, World, WriteStorage},
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
    let mut world = World::with_application_resources::<GameData<'_, '_>, _>(assets_dir)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(
            &mut world,
            NetworkBundle::<()>::new("127.0.0.1:23455".parse().unwrap()),
        )?
        .with(SpamReceiveSystem::new(), "rcv", &[]);
    let mut game = Application::build(State1, world)?
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
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world
            .create_entity()
            .with(NetConnection::<()>::new("127.0.0.1:3457".parse().unwrap()))
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
        for (conn,) in (&mut connections,).join() {
            if self.reader.is_none() {
                self.reader = Some(conn.register_reader());
            }

            for ev in conn.received_events(self.reader.as_mut().unwrap()) {
                count += 1;
                match ev {
                    _ => {}
                }
            }
        }
        info!("Received {} messages this frame", count);
    }
}

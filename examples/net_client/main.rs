extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::Result;
use amethyst::core::Time;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::ecs::{Read,Write,ReadStorage,WriteStorage,Join,System};
use amethyst::network::*;
use amethyst::prelude::*;
use amethyst::shrev::ReaderId;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

fn main() -> Result<()>{
    amethyst::start_logger(Default::default());
    let game_data = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<()>::new_client(
            "127.0.0.1",
            Some(3455 as u16),
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
impl<'a,'b> State<GameData<'a,'b>> for State1 {
    fn on_start(&mut self, mut data: StateData<GameData>) {
        data.world.create_entity().with(NetConnection::<()>::new("127.0.0.1:3456".parse().unwrap())).build();
    }
    
    fn update(&mut self, mut data: StateData<GameData>) -> Trans<GameData<'a,'b>> {
        data.data.update(&mut data.world);
        Trans::None
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
    type SystemData = (
        WriteStorage<'a, NetConnection<()>>,
        Read<'a, Time>,
    );
    fn run(&mut self, (mut connections, time): Self::SystemData) {
        for (mut conn,) in (&mut connections,).join() {
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
            //send_to_all(ev, &mut send_buf, &pool);
            conn.send_buffer.single_write(ev);
        }
       }
    }
}

/*
/// A simple system reading received events.
/// Used to see events sent by the net_echo_server example.
struct ReaderSystem {
    pub reader: Option<ReaderId<NetSourcedEvent<()>>>,
}

impl ReaderSystem {
    pub fn new() -> Self {
        ReaderSystem { reader: None }
    }
}

impl<'a> System<'a> for ReaderSystem {
    type SystemData = (FetchMut<'a, NetReceiveBuffer<()>>,);
    fn run(&mut self, (mut rcv,): Self::SystemData) {
        if self.reader.is_none() {
            self.reader = Some(rcv.buf.register_reader());
        }
        for ev in rcv.buf.read(self.reader.as_mut().unwrap()) {
            match ev.event {
                NetEvent::TextMessage { ref msg } => println!("Received: {}", msg),
                _ => {}
            }
        }
    }
}*/

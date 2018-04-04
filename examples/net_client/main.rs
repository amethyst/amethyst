extern crate amethyst;
#[macro_use]
extern crate log;

use std::time::Duration;

use amethyst::Result;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::Time;
use amethyst::network::*;
use amethyst::prelude::*;
use amethyst::shrev::EventChannel;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;

use amethyst::ecs::{Fetch, FetchMut, System};

use amethyst::network::NetworkClientBundle;

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let game = Application::build("", State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .with_bundle(
            NetworkClientBundle::<()>::new("127.0.0.1", Some(3455 as u16), vec![Box::new(FilterConnected)], false)
                .with_connect(SocketAddr::new(
                    IpAddr::from_str("127.0.0.1").unwrap(),
                    3456 as u16,
                )),
        )?
        .with(SpamSystem::new(), "spam_system", &[]);

    Ok(game.build()?.run())
}

pub struct State1;

impl State for State1 {}

struct SpamSystem{
    count: i32,
}

impl SpamSystem{
    pub fn new() -> Self{
        SpamSystem{
            count: 0,
        }
    }
}

impl<'a> System<'a> for SpamSystem {
    type SystemData = (
        FetchMut<'a, NetSendBuffer<()>>,
        Fetch<'a, NetConnectionPool>,
        Fetch<'a, Time>,
    );
    fn run(&mut self, (mut send_buf, pool, time): Self::SystemData) {
        self.count = 0;
        for i in 0..10000{
            self.count += 1;
            let ev = NetEvent::TextMessage {
                msg: format!("CL: frame:{},abs_time:{},c:{}",time.frame_number(), time.absolute_time_seconds(),self.count),
            };
            send_to_all(ev, &mut send_buf, &pool);
        }
    }
}

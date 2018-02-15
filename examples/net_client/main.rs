extern crate amethyst;
#[macro_use]
extern crate log;

use std::time::Duration;

use amethyst::Result;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::prelude::*;
use amethyst::network::*;
use amethyst::shrev::EventChannel;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut client = NetClientSystem::<()>::new("127.0.0.1",4545 as u16).expect("Failed to create NetClientSystem");
    client.connect(SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(),4546 as u16));

    let game = Application::build("",State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).with(client,"net_client_system",&[])
        .with_resource(NetSendBuffer::<()>::new())
        .with_resource(NetReceiveBuffer::<()>::new());

    Ok(
        game.build()?.run(),
    )
}

pub struct State1;

impl State for State1 {
}
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

use amethyst::network::NetworkClientBundle;

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let game = Application::build("",State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).with_bundle(NetworkClientBundle::<()>::new("127.0.0.1",Some(3455 as u16),vec![],false).with_connect(SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(),3456 as u16)))?;

    Ok(
        game.build()?.run(),
    )
}

pub struct State1;

impl State for State1 {
}
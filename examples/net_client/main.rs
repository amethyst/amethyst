use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, Time},
    ecs::{Read, System, Write},
    network::simulation::{
        laminar::{LaminarNetworkBundle, LaminarSocket},
        NetworkSimulationResource, NetworkSimulationTime,
    },
    prelude::*,
    utils::application_root_dir,
    Result,
};
use log::info;
use std::time::Duration;

// You'll likely want to use a type alias for any place you specify the NetworkResource<T> so
// that, if changed, it will only need to be changed in one place.
type NetworkResourceImpl = NetworkSimulationResource<LaminarSocket>;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let socket = LaminarSocket::bind("0.0.0.0:3455").expect("Should bind");

    let server_addr = "127.0.0.1:3457".parse()?;
    let assets_dir = application_root_dir()?.join("./");

    let mut net = NetworkSimulationResource::new_client(server_addr);
    net.set_socket(socket);

    let game_data = GameDataBuilder::default()
        .with_bundle(LaminarNetworkBundle)?
        .with(SpamSystem::new(), "spam", &[]);
    let mut game = Application::build(assets_dir, GameState)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .with_resource(net)
        .build(game_data)?;
    game.run();
    Ok(())
}
/// Default empty state
pub struct GameState;
impl SimpleState for GameState {}

/// A simple system that sends a ton of messages to all connections.
/// In this case, only the server is connected.
struct SpamSystem;

impl SpamSystem {
    pub fn new() -> Self {
        SpamSystem {}
    }
}

impl<'a> System<'a> for SpamSystem {
    type SystemData = (
        Read<'a, NetworkSimulationTime>,
        Read<'a, Time>,
        Write<'a, NetworkResourceImpl>,
    );
    fn run(&mut self, (sim_time, time, mut net): Self::SystemData) {
        // Use method `sim_time.sim_frames_to_run()` to determine if the system should send a
        // message this frame. If, for example, the ECS frame rate is slower than the simulation
        // frame rate, this code block will run until it catches up with the expected simulation
        // frame number.
        for frame in sim_time.sim_frames_to_run() {
            info!("Sending message for sim frame {}.", frame);
            let payload = format!(
                "CL: sim_frame:{},abs_time:{}",
                frame,
                time.absolute_time_seconds()
            );
            net.send(payload.as_bytes());
        }
    }
}

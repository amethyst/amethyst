extern crate amethyst;

use amethyst::{
    core::frame_limiter::FrameRateLimitStrategy, prelude::*, utils::application_root_dir,
};

use crate::{
    event::{MyExtendedStateEvent, MyExtendedStateEventReader},
    state::GameplayState,
};

mod event;
mod state;
mod system;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("examples/events_custom_state_event/assets");

    let game_data = GameDataBuilder::default().with(system::IncreaseGameDifficultySystem, "", &[]);

    let mut game = CoreApplication::<_, MyExtendedStateEvent, MyExtendedStateEventReader>::build(
        assets_dir,
        GameplayState::default(),
    )?
    .with_frame_limit(FrameRateLimitStrategy::Sleep, 1)
    .build(game_data)?;

    game.run();
    Ok(())
}

use std::{env, path::Path, sync::Arc};

use log::debug;
use rayon::ThreadPoolBuilder;
use winit::Event;

use crate::{
    assets::Loader,
    callback_queue::CallbackQueue,
    core::{
        frame_limiter::FrameLimiter,
        shrev::EventChannel,
        timing::{Stopwatch, Time},
        ArcThreadPool, Named,
    },
    ecs::{World, WorldExt},
    game_data::DataDispose,
    state::TransEvent,
    state_event::StateEvent,
    ui::UiEvent,
};

/// Extends the `World` to be initialized with resources to run an Amethyst application.
///
/// # Examples
///
/// ```rust,edition2018,no_run
/// use amethyst::{
///     ecs::World,
///     utils::application_root_dir,
///     AmethystWorldExt, Application, SimpleState, GameData, GameDataBuilder,
/// };
///
/// # struct NullState;
/// # impl SimpleState for NullState {}
///
/// fn main() -> amethyst::Result<()> {
///     let app_root = application_root_dir()?;
///     let assets_dir = app_root.join("examples/assets/");
///     let world = World::with_application_resources::<GameData<'_, '_>, _>(assets_dir)?;
///
///     let mut game_data = GameDataBuilder::default();
///
///     let mut game = Application::new(NullState, game_data, world)?;
///     game.run();
///     Ok(())
/// }
/// ```
pub trait AmethystWorldExt: private::Sealed {
    /// Returns a `World` with application resources.
    fn with_application_resources<T, P>(assets_dir: P) -> crate::Result<World>
    where
        T: DataDispose + 'static,
        P: AsRef<Path>;
}

// Disallow `AmethystWorldExt` from being implemented by other types.
mod private {
    use crate::ecs::World;

    pub trait Sealed {}
    impl Sealed for World {}
}

impl AmethystWorldExt for World {
    fn with_application_resources<T, P>(assets_dir: P) -> crate::Result<World>
    where
        T: DataDispose + 'static,
        P: AsRef<Path>,
    {
        let mut world = World::new();

        let thread_count: Option<usize> = env::var("AMETHYST_NUM_THREADS")
            .as_ref()
            .map(|s| {
                s.as_str()
                    .parse()
                    .expect("AMETHYST_NUM_THREADS was provided but is not a valid number!")
            })
            .ok();

        let thread_pool_builder = ThreadPoolBuilder::new();
        #[cfg(feature = "profiler")]
        let thread_pool_builder = thread_pool_builder.start_handler(|_index| {
            register_thread_with_profiler();
        });
        let pool: ArcThreadPool;
        if let Some(thread_count) = thread_count {
            debug!("Running Amethyst with fixed thread pool: {}", thread_count);
            pool = thread_pool_builder
                .num_threads(thread_count)
                .build()
                .map(Arc::new)?;
        } else {
            pool = thread_pool_builder.build().map(Arc::new)?;
        }
        world.insert(Loader::new(assets_dir.as_ref().to_owned(), pool.clone()));
        world.insert(pool);
        world.insert(EventChannel::<Event>::with_capacity(2000));
        world.insert(EventChannel::<UiEvent>::with_capacity(40));
        world.insert(EventChannel::<TransEvent<T, StateEvent>>::with_capacity(2));
        world.insert(FrameLimiter::default());
        world.insert(Stopwatch::default());
        world.insert(Time::default());
        world.insert(CallbackQueue::default());

        world.register::<Named>();

        Ok(world)
    }
}

//! Engine context passed into the active game state.

use assets::{Allocator, Loader};
use ecs::World;
use rayon::ThreadPool;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use winit::EventsLoop;

/// User-facing engine handle.
pub struct Engine {
    /// Asset manager.
    pub assets: Loader,
    /// Current delta time value.
    pub delta: Duration,
    /// Thread pool.
    pub pool: Arc<ThreadPool>,
    /// World.
    pub world: World,
}

impl Engine {
    /// Creates a new engine context.
    pub(crate) fn new<P>(base_path: P, pool: Arc<ThreadPool>, world: World) -> Self
        where P: AsRef<Path>
    {
        let alloc = Allocator::new();
        let loader = Loader::new(&alloc, base_path.as_ref(), pool.clone());

        Engine {
            assets: loader,
            delta: Duration::from_secs(0),
            pool: pool,
            world: world,
        }
    }
}

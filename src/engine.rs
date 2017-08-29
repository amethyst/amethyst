//! Engine context passed into the active game state.

use std::sync::Arc;
use std::time::Duration;

use rayon::ThreadPool;

use ecs::World;

/// User-facing engine handle.
pub struct Engine {
    /// Current delta time value.
    pub delta: Duration,
    /// Thread pool.
    pub pool: Arc<ThreadPool>,
    /// World.
    pub world: World,
}

impl Engine {
    /// Creates a new engine context.
    pub(crate) fn new(pool: Arc<ThreadPool>, world: World) -> Self {
        Engine {
            delta: Duration::from_secs(0),
            pool: pool,
            world: world,
        }
    }
}

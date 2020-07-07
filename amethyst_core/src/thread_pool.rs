use amethyst_error::Error;
#[cfg(not(no_threading))]
use std::sync::Arc;

#[cfg(not(no_threading))]
#[derive(Debug, Clone)]
pub struct ThreadPool {
    pool: Arc<rayon::ThreadPool>,
}

#[cfg(no_threading)]
#[derive(Debug, Clone)]
pub struct ThreadPool {}

impl ThreadPool {
    #[cfg(not(no_threading))]
    pub fn new(thread_count: Option<usize>) -> Result<Self, Error> {
        use rayon::ThreadPoolBuilder;
        let thread_pool_builder = ThreadPoolBuilder::new();
        #[cfg(feature = "profiler")]
        let thread_pool_builder = thread_pool_builder.start_handler(|_index| {
            register_thread_with_profiler();
        });
        let pool = if let Some(thread_count) = thread_count {
            thread_pool_builder.num_threads(thread_count).build()
        } else {
            thread_pool_builder.build()
        };
        Ok(Self {
            pool: Arc::new(pool?),
        })
    }

    #[cfg(no_threading)]
    pub fn new(_: Option<usize>) -> Result<Self, Error> {
        Ok(Self {})
    }

    #[cfg(not(no_threading))]
    pub fn rayon(&self) -> Arc<rayon::ThreadPool> {
        self.pool.clone()
    }

    pub fn spawn<OP>(&self, op: OP)
    where
        OP: FnOnce() + Send + 'static,
    {
        #[cfg(not(no_threading))]
        self.pool.spawn(op);
        #[cfg(no_threading)]
        op();
    }
}

use specs::prelude::*;

/// Simplified DispatcherBuilder
///
/// See `DispatcherBuilder` and `GameDataBuilder` for more details.
pub trait SimpleDispatcherBuilder<'a, 'b, 'c>: Sized {
    /// Adds a new system with a given name and a list of dependencies.
    /// Please note that the dependency should be added before
    /// you add the depending system.
    ///
    /// If you want to register systems which can not be specified as
    /// dependencies, you can use `""` as their name, which will not panic
    /// (using another name twice will).
    ///
    /// Same as [`add()`](struct.DispatcherBuilder.html#method.add), but
    /// returns `self` to enable method chaining.
    ///
    /// # Panics
    ///
    /// * if the specified dependency does not exist
    /// * if a system with the same name was already registered.
    fn with<T>(mut self, system: T, name: &'c str, dep: &[&'c str]) -> Self
    where
        T: for<'d> System<'d> + Send + 'a + 'c,
    {
        self.add(system, name, dep);

        self
    }

    /// Adds a new system with a given name and a list of dependencies.
    /// Please note that the dependency should be added before
    /// you add the depending system.
    ///
    /// If you want to register systems which can not be specified as
    /// dependencies, you can use `""` as their name, which will not panic
    /// (using another name twice will).
    ///
    /// # Panics
    ///
    /// * if the specified dependency does not exist
    /// * if a system with the same name was already registered.
    fn add<T>(&mut self, system: T, name: &'c str, dep: &[&'c str])
    where
        T: for<'d> System<'d> + Send + 'a + 'c;

    /// Adds a new thread local system.
    ///
    /// Please only use this if your struct is not `Send` and `Sync`.
    ///
    /// Thread-local systems are dispatched in-order.
    ///
    /// Same as
    /// [`add_thread_local()`](struct.DispatcherBuilder.html#method.add_thread_local),
    /// but returns `self` to enable method chaining.
    fn with_thread_local<T>(mut self, system: T) -> Self
    where
        T: for<'d> RunNow<'d> + 'b + 'c,
    {
        self.add_thread_local(system);

        self
    }

    /// Adds a new thread local system.
    ///
    /// Please only use this if your struct is not `Send` and `Sync`.
    ///
    /// Thread-local systems are dispatched in-order.
    fn add_thread_local<T>(&mut self, system: T)
    where
        T: for<'d> RunNow<'d> + 'b + 'c;

    /// Inserts a barrier which assures that all systems
    /// added before the barrier are executed before the ones
    /// after this barrier.
    ///
    /// Does nothing if there were no systems added
    /// since the last call to `add_barrier()`/`with_barrier()`.
    ///
    /// Thread-local systems are not affected by barriers;
    /// they're always executed at the end.
    ///
    /// Same as
    /// [`add_barrier()`](struct.DispatcherBuilder.html#method.add_barrier),
    /// but returns `self` to enable method chaining.
    fn with_barrier(mut self) -> Self {
        self.add_barrier();

        self
    }

    /// Inserts a barrier which assures that all systems
    /// added before the barrier are executed before the ones
    /// after this barrier.
    ///
    /// Does nothing if there were no systems added
    /// since the last call to `add_barrier()`/`with_barrier()`.
    ///
    /// Thread-local systems are not affected by barriers;
    /// they're always executed at the end.
    fn add_barrier(&mut self);
}

impl<'a, 'b, 'c> SimpleDispatcherBuilder<'a, 'b, 'c> for DispatcherBuilder<'a, 'b> {
    fn add<T>(&mut self, system: T, name: &'c str, dep: &[&'c str])
    where
        T: for<'d> System<'d> + Send + 'a + 'c,
    {
        DispatcherBuilder::add(self, system, name, dep);
    }

    fn add_thread_local<T>(&mut self, system: T)
    where
        T: for<'d> RunNow<'d> + 'b + 'c,
    {
        DispatcherBuilder::add_thread_local(self, system);
    }

    fn add_barrier(&mut self) {
        DispatcherBuilder::add_barrier(self);
    }
}

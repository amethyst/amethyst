# How to Define State Dispatcher: External

This guide explains how to define a state-specific dispatcher whose systems are passed in externally. This is used when the list of systems is determined by user choices at runtime.

## Steps

1. Create a builder type for the state.

    This will be used to register the `System`s for the state-specific dispatcher.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    use amethyst::{ecs::prelude::*, prelude::*};

    /// Builder for the `CustomDispatcherState`.
    ///
    /// This allows you to specify which systems you want to run within the state.
    pub struct CustomDispatcherStateBuilder<'a, 'b> {
        /// State specific dispatcher.
        dispatcher_builder: DispatcherBuilder<'a, 'b>,
    }

    impl<'a, 'b> CustomDispatcherStateBuilder<'a, 'b> {
        /// Returns a new `CustomDispatcherStateBuilder`.
        pub fn new() -> Self {
            CustomDispatcherStateBuilder {
                dispatcher_builder: DispatcherBuilder::new(),
            }
        }

        /// Registers a `System` with the dispatcher builder.
        ///
        /// # Parameters
        ///
        /// * `system`: `System` to register.
        /// * `name`: Name to register the system with, used for dependency ordering.
        /// * `deps`: Names of systems that must run before this system.
        pub fn with<Sys>(mut self, system: Sys, name: &str, deps: &[&str]) -> Self
        where
            Sys: for<'c> System<'c> + Send + 'a,
        {
            self.dispatcher_builder.add(system, name, deps);
            self
        }

        /// Builds and returns the `CustomDispatcherState`.
        pub fn build(self) -> CustomDispatcherState<'a, 'b> {
            CustomDispatcherState::new(self.dispatcher_builder)
        }
    }
    #
    # pub struct CustomDispatcherState<'a, 'b> {
    #     /// State specific dispatcher builder.
    #     dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
    # }
    #
    # impl<'a, 'b> CustomDispatcherState<'a, 'b> {
    #     fn new(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
    #         unimplemented!()
    #     }
    # }
    ```

2. Add the `dispatcher_builder` and `dispatcher` fields to your state:

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    # use amethyst::{ecs::prelude::*, prelude::*};
    #
    pub struct CustomDispatcherState<'a, 'b> {
        /// State specific dispatcher builder.
        dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
        /// State specific dispatcher.
        dispatcher: Option<Dispatcher<'a, 'b>>,
    }

    impl<'a, 'b> CustomDispatcherState<'a, 'b> {
        fn new(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
            CustomDispatcherState {
                dispatcher_builder: Some(dispatcher_builder),
                dispatcher: None,
            }
        }
    }
    ```

3. Initialize the dispatcher `on_start(..)`.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    # use amethyst::{ecs::prelude::*, prelude::*};
    #
    # pub struct CustomDispatcherState<'a, 'b> {
    #     /// State specific dispatcher builder.
    #     dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
    #     /// State specific dispatcher.
    #     dispatcher: Option<Dispatcher<'a, 'b>>,
    # }
    #
    impl<'a, 'b, E> State<GameData<'a, 'b>, E> for CustomDispatcherState<'a, 'b>
    where
        E: Send + Sync + 'static,
    {
        fn on_start(&mut self, data: StateData<'_, GameData<'a, 'b>>) {
            if self.dispatcher.is_none() {
                let mut dispatcher = self
                    .dispatcher_builder
                    .take()
                    .expect(
                        "Expected `dispatcher_builder` to exist when `dispatcher` is not yet built.",
                    )
                    .build();
                dispatcher.setup(&mut data.world.res);
                self.dispatcher = Some(dispatcher);
            }
        }

        // ..
    }
    ```

    `System` resources are setup when the dispatcher is built. The `System#setup(..)` method needs access to the world's resources, which is made available to `State#on_start(..)`. Therefore, we need to defer building the actual `Dispatcher` until this method is called.

4. Run both the application and state-specific dispatchers during `update(..)`.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    # use amethyst::{ecs::prelude::*, prelude::*};
    #
    # pub struct CustomDispatcherState<'a, 'b> {
    #     /// State specific dispatcher builder.
    #     dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
    #     /// State specific dispatcher.
    #     dispatcher: Option<Dispatcher<'a, 'b>>,
    # }
    #
    impl<'a, 'b, E> State<GameData<'a, 'b>, E> for CustomDispatcherState<'a, 'b>
    where
        E: Send + Sync + 'static,
    {
        //..

        fn update(&mut self, data: StateData<'_, GameData<'a, 'b>>) -> Trans<GameData<'a, 'b>, E> {
            data.data.update(&data.world);
            self.dispatcher.as_mut().unwrap().dispatch(&data.world.res);

            Trans::Pop
        }
    }
    ```

    Now, any systems in the state-specific dispatcher will only run in this state.

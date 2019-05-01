# How to Define State Dispatcher: Internal

This guide explains how to define a state-specific dispatcher whose systems are determined by the state and is unaffected by input from a previous state.

## Steps

1. Add the `dispatcher` field to your state:

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    # use amethyst::{ecs::prelude::*, prelude::*};
    #
    pub struct CustomDispatcherState<'a, 'b> {
        /// State specific dispatcher.
        dispatcher: Option<Dispatcher<'a, 'b>>,
    }

    impl<'a, 'b> CustomDispatcherState<'a, 'b> {
        fn new() -> Self {
            CustomDispatcherState {
                dispatcher: None,
            }
        }
    }
    ```

2. Initialize the dispatcher `on_start(..)`.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    # use amethyst::{ecs::prelude::*, prelude::*};
    #
    # pub struct CustomDispatcherState<'a, 'b> {
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
                let mut dispatcher_builder = DispatcherBuilder::new();

                // Register systems here.
                // dispatcher_builder.add(MySystem, "my_system", &["dep"]);

                // If you would like to register bundles,
                // you may do so here.
                // MyBundle::new()
                //     .build(&mut dispatcher_builder)
                //     .expect("Failed to register `MyBundle`.");

                let mut dispatcher = dispatcher_builder.build();
                dispatcher.setup(&mut data.world.res);
                self.dispatcher = Some(dispatcher);
            }
        }

        // ..
    }
    ```

3. Run both the application and state-specific dispatchers during `update(..)`.

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

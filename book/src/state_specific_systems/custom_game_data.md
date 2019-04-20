# Custom `GameData`

So far we've been using the `Amethyst` supplied `GameData` struct to handle
our `System`s. This works well for smaller games and demos, but once we
start building a larger game, we will quickly realise we need to 
manipulate the `System` dispatch based on game `State`, or we need to pass
data between `State`s that aren't `Send + Sync` which can't be added to `World`.

The solution to our troubles here is to create a custom `GameData` structure 
to house what we need that can not be added to `World`.

In this tutorial we will look at how one could structure a `Paused` `State`, 
which disables the game logic, only leaving a few core systems running that 
are essential (like rendering, input and UI).

Let's start by creating the `GameData` structure:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::prelude::Dispatcher;
#
pub struct CustomGameData<'a, 'b> {
    core_dispatcher: Dispatcher<'a, 'b>,
    running_dispatcher: Dispatcher<'a, 'b>,
}
```

We also add a utility function for performing dispatch:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::prelude::{Dispatcher, World};
#
# pub struct CustomGameData<'a, 'b> {
#     core_dispatcher: Dispatcher<'a, 'b>,
#     running_dispatcher: Dispatcher<'a, 'b>,
# }
#
impl<'a, 'b> CustomGameData<'a, 'b> {
    /// Update game data
    pub fn update(&mut self, world: &World, running: bool) {
        if running {
            self.running_dispatcher.dispatch(&world.res);
        }
        self.core_dispatcher.dispatch(&world.res);
    }
}
```

To be able to use this structure with `Amethyst`s `Application` we need to create
a builder that implements `DataInit`. This is the only requirement placed on the
`GameData` structure.

```rust,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::ecs::prelude::{Dispatcher, DispatcherBuilder, System, World};
# use amethyst::core::SystemBundle;
# use amethyst::{Error, DataInit};
#
# pub struct CustomGameData<'a, 'b> {
#     core_dispatcher: Dispatcher<'a, 'b>,
#     running_dispatcher: Dispatcher<'a, 'b>,
# }
#
use amethyst::core::ArcThreadPool;

pub struct CustomGameDataBuilder<'a, 'b> {
    pub core: DispatcherBuilder<'a, 'b>,
    pub running: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> Default for CustomGameDataBuilder<'a, 'b> {
    fn default() -> Self {
        CustomGameDataBuilder::new()
    }
}

impl<'a, 'b> CustomGameDataBuilder<'a, 'b> {
    pub fn new() -> Self {
        CustomGameDataBuilder {
            core: DispatcherBuilder::new(),
            running: DispatcherBuilder::new(),
        }
    }

    pub fn with_base_bundle<B>(mut self, bundle: B) -> Result<Self, Error>
    where
        B: SystemBundle<'a, 'b>,
    {
        bundle.build(&mut self.core)?;
        Ok(self)
    }

    pub fn with_running<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Self
    where
        for<'c> S: System<'c> + Send + 'a,
    {
        self.running.add(system, name, dependencies);
        self
    }
}

impl<'a, 'b> DataInit<CustomGameData<'a, 'b>> for CustomGameDataBuilder<'a, 'b> {
    fn build(self, world: &mut World) -> CustomGameData<'a, 'b> {
        // Get a handle to the `ThreadPool`.
        let pool = world.read_resource::<ArcThreadPool>().clone();

        let mut core_dispatcher = self.core.with_pool(pool.clone()).build();
        let mut running_dispatcher = self.running.with_pool(pool.clone()).build();
        core_dispatcher.setup(&mut world.res);
        running_dispatcher.setup(&mut world.res);

        CustomGameData { core_dispatcher, running_dispatcher }
    }
}
```

We can now use `CustomGameData` in place of the provided `GameData` when building
our `Application`, but first we should create some `State`s.

```rust,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::ecs::prelude::{Dispatcher, World};
# use amethyst::prelude::{State, StateData, StateEvent, Trans};
# use amethyst::input::{is_close_requested, is_key_down};
# use amethyst::renderer::VirtualKeyCode;
#
# pub struct CustomGameData<'a, 'b> {
#     core_dispatcher: Dispatcher<'a, 'b>,
#     running_dispatcher: Dispatcher<'a, 'b>,
# }
#
# impl<'a, 'b> CustomGameData<'a, 'b> {
#     /// Update game data
#     pub fn update(&mut self, world: &World, running: bool) {
#         if running {
#             self.running_dispatcher.dispatch(&world.res);
#         }
#         self.core_dispatcher.dispatch(&world.res);
#     }
# }
#
# fn initialise(world: &World) {}
# fn create_paused_ui(world: &World) {}
# fn delete_paused_ui(world: &World) {}
#
struct Main;
struct Paused;

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for Paused {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        create_paused_ui(data.world);
    }

    fn handle_event(
        &mut self,
        data: StateData<CustomGameData>,
        event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::Space) {
                delete_paused_ui(data.world);
                Trans::Pop
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<CustomGameData>) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.data.update(&data.world, false); // false to say we should not dispatch running
        Trans::None
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for Main {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        initialise(data.world);
    }

    fn handle_event(
        &mut self,
        _: StateData<CustomGameData>,
        event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::Space) {
                Trans::Push(Box::new(Paused))
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<CustomGameData>) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.data.update(&data.world, true); // true to say we should dispatch running
        Trans::None
    }
}
```

The only thing that remains now is to use our `CustomGameDataBuilder` when building the
`Application`.

```rust,ignore
let game_data = CustomGameDataBuilder::default()
    .with_running(ExampleSystem, "example_system", &[])
    .with_base_bundle(TransformBundle::new())?
    .with_base_bundle(UiBundle::<String, String>::new())?
    .with_base_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?
    .with_base_bundle(InputBundle::<String, String>::new())?;

let mut game = Application::new(resources_directory, Main, game_data)?;
game.run();
```

Those are the basics of creating a custom `GameData` structure. Now get out there and
build your game!


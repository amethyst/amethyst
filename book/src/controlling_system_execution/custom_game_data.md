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

```rust
# extern crate amethyst;
# use amethyst::ecs::Dispatcher;
# 
pub struct CustomGameData<'a, 'b> {
    core_dispatcher: Option<Dispatcher>,
    running_dispatcher: Option<Dispatcher>,
}
```

We also add a utility function for performing dispatch:

```rust
# extern crate amethyst;
# use amethyst::ecs::{Dispatcher, World};
# 
# pub struct CustomGameData<'a, 'b> {
#   core_dispatcher: Option<Dispatcher>,
#   running_dispatcher: Option<Dispatcher>,
# }
# 
impl<'a, 'b> CustomGameData<'a, 'b> {
    /// Update game data
    pub fn update(&mut self, world: &World, running: bool) {
        if running {
            if let Some(dispatcher) = self.running_dispatcher.as_mut() {
                dispatcher.dispatch(&world);
            }
        }
        if let Some(dispatcher) = self.core_dispatcher.as_mut() {
            dispatcher.dispatch(&world);
        }
    }
}
```

To be able to use this structure with `Amethyst`s `Application` we need to create
a builder that implements `DataInit`, as well as implement `DataDispose` for our
`GameData` structure. These are the only requirements placed on the
`GameData` structure.

```rust
# extern crate amethyst;
# use amethyst::core::SystemBundle;
# use amethyst::ecs::{Dispatcher, DispatcherBuilder, System, World};
# use amethyst::{DataDispose, DataInit, Error};
# 
# pub struct CustomGameData<'a, 'b> {
#   core_dispatcher: Option<Dispatcher>,
#   running_dispatcher: Option<Dispatcher>,
# }
# 
use amethyst::core::ArcThreadPool;

pub struct CustomDispatcherBuilder {
    pub core: DispatcherBuilder,
    pub running: DispatcherBuilder,
}

impl<'a, 'b> Default for CustomDispatcherBuilder {
    fn default() -> Self {
        CustomDispatcherBuilder::new()
    }
}

impl<'a, 'b> CustomDispatcherBuilder {
    pub fn new() -> Self {
        CustomDispatcherBuilder {
            core: DispatcherBuilder::new(),
            running: DispatcherBuilder::new(),
        }
    }

    pub fn with_base_bundle<B>(mut self, world: &mut World, bundle: B) -> Result<Self, Error>
    where
        B: SystemBundle<'a, 'b>,
    {
        bundle.build(world, &mut self.core)?;
        Ok(self)
    }

    pub fn with_running<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Self
    where
        for<'c> S: System + Send + 'a,
    {
        self.running.add(system, name, dependencies);
        self
    }
}

impl<'a, 'b> DataInit<CustomGameData<'a, 'b>> for CustomDispatcherBuilder {
    fn build(self, world: &mut World) -> CustomGameData<'a, 'b> {
        // Get a handle to the `ThreadPool`.
        let pool = (*resources.get::<ArcThreadPool>()).clone();

        let mut core_dispatcher = self.core.with_pool(pool.clone()).build();
        let mut running_dispatcher = self.running.with_pool(pool.clone()).build();
        core_dispatcher.setup(world);
        running_dispatcher.setup(world);

        let core_dispatcher = Some(core_dispatcher);
        let running_dispatcher = Some(running_dispatcher);

        CustomGameData {
            core_dispatcher,
            running_dispatcher,
        }
    }
}

impl<'a, 'b> DataDispose for CustomGameData<'a, 'b> {
    // We dispose each dispatcher owned by the `CustomGameData` structure.
    fn dispose(&mut self, world: &mut World) {
        if let Some(dispatcher) = self.core_dispatcher.take() {
            dispatcher.dispose(world);
        }
        if let Some(dispatcher) = self.running_dispatcher.take() {
            dispatcher.dispose(world);
        }
    }
}
```

We can now use `CustomGameData` in place of the provided `GameData` when building
our `Application`, but first we should create some `State`s.

```rust
# extern crate amethyst;
# use amethyst::ecs::{Dispatcher, World};
# use amethyst::input::{is_close_requested, is_key_down, VirtualKeyCode};
# use amethyst::prelude::*;
# 
# pub struct CustomGameData<'a, 'b> {
#   core_dispatcher: Option<Dispatcher>,
#   running_dispatcher: Option<Dispatcher>,
# }
# 
# impl<'a, 'b> CustomGameData<'a, 'b> {
#   /// Update game data
#   pub fn update(&mut self, world: &World, running: bool) {
#       if running {
#           if let Some(dispatcher) = self.running_dispatcher.as_mut() {
#               dispatcher.dispatch(&world);
#           }
#       }
#       if let Some(dispatcher) = self.core_dispatcher.as_mut() {
#           dispatcher.dispatch(&world);
#       }
#   }
# }
# 
# fn initialize(world: &World) {}
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

    fn update(
        &mut self,
        data: StateData<CustomGameData>,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.data.update(&data.world, false); // false to say we should not dispatch running
        Trans::None
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for Main {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        initialize(data.world);
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

    fn update(
        &mut self,
        data: StateData<CustomGameData>,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.data.update(&data.world, true); // true to say we should dispatch running
        Trans::None
    }
}
```

The only thing that remains now is to use our `CustomDispatcherBuilder` when building the
`Application`.

```rust
# extern crate amethyst;
# use amethyst::{
#   core::{transform::TransformBundle, SystemBundle},
#   ecs::{Dispatcher, DispatcherBuilder, World},
#   input::InputBundle,
#   prelude::*,
#   renderer::{
#       plugins::{RenderFlat2D, RenderToWindow},
#       types::DefaultBackend,
#       RenderingBundle,
#   },
#   ui::{RenderUi, UiBundle},
#   utils::application_root_dir,
#   DataDispose, DataInit, Error,
# };
# 
# pub struct CustomGameData<'a, 'b> {
#   core_dispatcher: Option<Dispatcher>,
#   running_dispatcher: Option<Dispatcher>,
# }
# 
# pub struct CustomDispatcherBuilder {
#   pub core: DispatcherBuilder,
#   pub running: DispatcherBuilder,
# }
# 
# impl<'a, 'b> Default for CustomDispatcherBuilder {
#   fn default() -> Self {
#       unimplemented!()
#   }
# }
# 
# impl<'a, 'b> CustomDispatcherBuilder {
#   pub fn new() -> Self {
#       unimplemented!()
#   }
#   pub fn with_base_bundle<B>(mut self, world: &mut World, bundle: B) -> Result<Self, Error>
#   where
#       B: SystemBundle<'a, 'b>,
#   {
#       unimplemented!()
#   }
# 
#   pub fn with_running<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Self
#   where
#       for<'c> S: System + Send + 'a,
#   {
#       unimplemented!()
#   }
# }
# 
# impl<'a, 'b> DataInit<CustomGameData<'a, 'b>> for CustomDispatcherBuilder {
#   fn build(self, world: &mut World) -> CustomGameData<'a, 'b> {
#       unimplemented!()
#   }
# }
# 
# impl<'a, 'b> DataDispose for CustomDispatcherBuilder {
#   fn dispose(&mut self, world: &mut World) {
#       unimplemented!()
#   }
# }
# 
# fn main() -> amethyst::Result<()> {
    let mut app_builder = Application::build(assets_directory, Main)?;
    let game_data = CustomDispatcherBuilder::default()
        .with_running(ExampleSystem, "example_system", &[])
        .with_base_bundle(
            &mut app_builder.world,
            RenderingBundle::<DefaultBackend>::new()
                // The RenderToWindow plugin provides all the scaffolding for opening a window and
                // drawing on it
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default()),
        )?
        .with_base_bundle(&mut app_builder.world, TransformBundle::new())?
        .with_base_bundle(&mut app_builder.world, UiBundle::new())?
        .with_base_bundle(
            &mut app_builder.world,
            InputBundle::new().with_bindings_from_file(key_bindings_path)?,
        )?;

    let mut game = app_builder.build(game_data)?;
    //game.run();
# }
```

Those are the basics of creating a custom `GameData` structure. Now get out there and
build your game!

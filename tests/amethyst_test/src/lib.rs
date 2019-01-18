#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! Test harness to support testing of Amethyst types, including:
//!
//! * `Bundle`
//! * `State`
//! * `System`
//! * Resource loading.
//! * Arbitrary types that `System`s use during processing.
//!
//! The test harness minimizes boilerplate code to set up an Amethyst `Application` with common
//! bundles, and can take in logic that is normally masked behind a number of layers through a thin
//! interface.
//!
//! # Usage
//!
//! The following shows a simple example of testing a `State`. More examples are in the
//! [Examples](#Examples) section.
//!
//! ```rust
//! # use std::marker::PhantomData;
//! #
//! # use amethyst_test::prelude::*;
//! # use amethyst::{
//! #     ecs::prelude::*,
//! #     prelude::*,
//! # };
//! #
//! # #[derive(Debug)]
//! # struct LoadResource;
//! #
//! # #[derive(Debug)]
//! # struct LoadingState;
//! #
//! # impl LoadingState {
//! #     fn new() -> Self {
//! #         LoadingState
//! #     }
//! # }
//! #
//! # impl<'a, 'b, E> State<GameData<'a, 'b>, E> for LoadingState
//! # where
//! #     E: Send + Sync + 'static,
//! # {
//! #     fn update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> Trans<GameData<'a, 'b>, E> {
//! #         data.data.update(&data.world);
//! #
//! #         data.world.add_resource(LoadResource);
//! #
//! #         Trans::Pop
//! #     }
//! # }
//! #
//! // #[test]
//! fn loading_state_adds_load_resource() {
//!     assert!(
//!         AmethystApplication::blank()
//!             .with_state(|| LoadingState::new())
//!             .with_assertion(|world| {
//!                 world.read_resource::<LoadResource>();
//!             })
//!             .run()
//!             .is_ok()
//!     );
//! }
//! #
//! # fn main() {
//! #     loading_state_adds_load_resource();
//! # }
//! ```
//!
//! The Amethyst application is initialized with one of the following functions, each providing a
//! different set of bundles:
//!
//! ```rust,no_run
//! use amethyst_test::prelude::*;
//!
//! #[test]
//! fn test_name() {
//!     // Start with no bundles
//!     AmethystApplication::blank();
//!
//!     // Start with the Transform, Input, and UI bundles
//!     // The type parameters here are the Axis and Action types for the `InputBundle` and
//!     // `UiBundle`.
//!     AmethystApplication::ui_base::<String, String>();
//!
//!     // Start with the Animation, Transform, and Render bundles.
//!     // If you want the Input and UI bundles, you can use the `.with_ui_bundles::<AX, AC>()`
//!     // method.
//!     let visibility = false; // Whether the window should be shown
//!     AmethystApplication::render_base("test_name", visibility);
//! }
//! ```
//!
//! Next, attach the logic you wish to test using the various `.with_*(..)` methods:
//!
//! ```rust,no_run
//! #[test]
//! fn test_name() {
//!     let visibility = false; // Whether the window should be shown
//!     AmethystApplication::render_base::<String, String, _>("test_name", visibility)
//!         .with_bundle(MyBundle::new())                // Registers a bundle.
//!         .with_bundle_fn(|| MyNonSendBundle::new())   // Registers a `!Send` bundle.
//!         .with_resource(MyResource::new())            // Adds a resource to the world.
//!         .with_system(MySystem::new(), "my_sys", &[]) // Registers a system with the main
//!                                                      // dispatcher.
//!
//!         // These are run in the order they are invoked.
//!         // You may invoke them multiple times.
//!         .with_setup(|world| { /* do something */ })
//!         .with_state(|| MyState::new())
//!         .with_effect(|world| { /* do something */ })
//!         .with_assertion(|world| { /* do something */ })
//!          // ...
//! }
//! ```
//!
//! Finally, call `.run()` to run the application. This returns `amethyst::Result<()>`, so you can
//! wrap it in an `assert!(..);`:
//!
//! ```rust,no_run
//! #[test]
//! fn test_name() {
//!     let visibility = false; // Whether the window should be shown
//!     assert!(
//!         AmethystApplication::render_base("test_name", visibility)
//!             // ...
//!             .run()
//!             .is_ok()
//!     );
//! }
//! ```
//!
//! # Examples
//!
//! Testing a bundle:
//!
//! ```rust
//! # use amethyst_test::prelude::*;
//! # use amethyst::{
//! #     core::bundle::{self, SystemBundle},
//! #     ecs::prelude::*,
//! #     prelude::*,
//! # };
//! #
//! # #[derive(Debug)]
//! # struct ApplicationResource;
//! #
//! # #[derive(Debug)]
//! # struct MySystem;
//! #
//! # impl<'s> System<'s> for MySystem {
//! #     type SystemData = ReadExpect<'s, ApplicationResource>;
//! #
//! #     fn run(&mut self, _: Self::SystemData) {}
//! #
//! #     fn setup(&mut self, res: &mut Resources) {
//! #         Self::SystemData::setup(res);
//! #         res.insert(ApplicationResource);
//! #     }
//! # }
//! #
//! # #[derive(Debug)]
//! # struct MyBundle;
//! # impl<'a, 'b> SystemBundle<'a, 'b> for MyBundle {
//! #     fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
//! #         builder.add(MySystem, "my_system", &[]);
//! #         Ok(())
//! #     }
//! # }
//! #
//! // #[test]
//! fn bundle_registers_system_with_resource() {
//!     assert!(
//!         AmethystApplication::blank()
//!             .with_bundle(MyBundle)
//!             .with_assertion(|world| { world.read_resource::<ApplicationResource>(); })
//!             .run()
//!             .is_ok()
//!     );
//! }
//! #
//! # fn main() {
//! #     bundle_registers_system_with_resource();
//! # }
//! ```
//!
//! Testing a system:
//!
//! ```rust
//! # use amethyst_test::prelude::*;
//! # use amethyst::{
//! #     ecs::prelude::*,
//! #     prelude::*,
//! # };
//! #
//! # struct MyComponent(pub i32);
//! #
//! # impl Component for MyComponent {
//! #     type Storage = DenseVecStorage<Self>;
//! # }
//! #
//! # #[derive(Debug)]
//! # struct MySystem;
//! #
//! # impl<'s> System<'s> for MySystem {
//! #     type SystemData = WriteStorage<'s, MyComponent>;
//! #
//! #     fn run(&mut self, mut my_component_storage: Self::SystemData) {
//! #         for mut my_component in (&mut my_component_storage).join() {
//! #             my_component.0 += 1
//! #         }
//! #     }
//! # }
//! #
//! // #[test]
//! fn system_increases_component_value_by_one() {
//!     assert!(
//!         AmethystApplication::blank()
//!             .with_system(MySystem, "my_system", &[])
//!             .with_effect(|world| {
//!                 let entity = world.create_entity().with(MyComponent(0)).build();
//!                 world.add_resource(EffectReturn(entity));
//!             })
//!             .with_assertion(|world| {
//!                 let entity = world.read_resource::<EffectReturn<Entity>>().0.clone();
//!
//!                 let my_component_storage = world.read_storage::<MyComponent>();
//!                 let my_component = my_component_storage
//!                     .get(entity)
//!                     .expect("Entity should have a `MyComponent` component.");
//!
//!                 // If the system ran, the value in the `MyComponent` should be 1.
//!                 assert_eq!(1, my_component.0);
//!             })
//!             .run()
//!             .is_ok()
//!     );
//! }
//! #
//! # fn main() {
//! #     system_increases_component_value_by_one();
//! # }
//! ```
//!
//! Testing a System in a custom dispatcher. This is useful when your system must run *after* some
//! setup has been done:
//!
//! ```rust
//! # use amethyst_test::prelude::*;
//! # use amethyst::{
//! #     ecs::prelude::*,
//! #     prelude::*,
//! # };
//! #
//! # // !Default
//! # struct MyResource(pub i32);
//! #
//! # #[derive(Debug)]
//! # struct MySystem;
//! #
//! # impl<'s> System<'s> for MySystem {
//! #     type SystemData = WriteExpect<'s, MyResource>;
//! #
//! #     fn run(&mut self, mut my_resource: Self::SystemData) {
//! #         my_resource.0 += 1
//! #     }
//! # }
//! #
//! // #[test]
//! fn system_increases_resource_value_by_one() {
//!     assert!(
//!         AmethystApplication::blank()
//!             .with_setup(|world| {
//!                 world.add_resource(MyResource(0));
//!             })
//!             .with_system_single(MySystem, "my_system", &[])
//!             .with_assertion(|world| {
//!                 let my_resource = world.read_resource::<MyResource>();
//!
//!                 // If the system ran, the value in the `MyResource` should be 1.
//!                 assert_eq!(1, my_resource.0);
//!             })
//!             .run()
//!             .is_ok()
//!     );
//! }
//! #
//! # fn main() {
//! #     system_increases_resource_value_by_one();
//! # }
//! ```

pub(crate) use crate::system_injection_bundle::SystemInjectionBundle;
pub use crate::{
    amethyst_application::{AmethystApplication, HIDPI, SCREEN_HEIGHT, SCREEN_WIDTH},
    effect_return::EffectReturn,
    fixture::{MaterialAnimationFixture, SpriteRenderAnimationFixture},
    game_update::GameUpdate,
    state::{
        CustomDispatcherState, CustomDispatcherStateBuilder, FunctionState, PopState,
        SequencerState,
    },
};

mod amethyst_application;
mod effect_return;
mod fixture;
mod game_update;
pub mod prelude;
mod state;
mod system_injection_bundle;

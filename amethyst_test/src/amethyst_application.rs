use std::{any::Any, marker::PhantomData, panic, path::PathBuf, sync::Mutex};

use amethyst::{
    self,
    core::{transform::TransformBundle, EventReader, RunNowDesc, SystemBundle, SystemDesc},
    ecs::prelude::*,
    error::Error,
    input::{BindingTypes, InputBundle},
    prelude::*,
    shred::Resource,
    ui::UiBundle,
    utils::application_root_dir,
    window::ScreenDimensions,
    StateEventReader,
};
use derivative::Derivative;
use lazy_static::lazy_static;

use crate::{
    CustomDispatcherStateBuilder, FunctionState, GameUpdate, SequencerState,
    SystemDescInjectionBundle, SystemInjectionBundle, ThreadLocalInjectionBundle,
};

type BundleAddFn = Box<
    dyn FnOnce(
        GameDataBuilder<'static, 'static>,
    ) -> Result<GameDataBuilder<'static, 'static>, Error>,
>;
// Hack: Ideally we want a `SendBoxFnOnce`. However implementing it got too crazy:
//
// * When taking in `ApplicationBuilder<StateLocal>` as a parameter, I couldn't get the type
//   parameters to be happy. `StateLocal` had to change depending on the first state, but it
//   couldn't be consolidated with `T`.
// * When using `SendBoxFnOnce<'w, (&'w mut World,)>`, the lifetime parameter for the function and
//   the `World` could not agree &mdash; you can't coerce a `SendBoxFnOnce<'longer>` into a
//   `SendBoxFnOnce<'shorter>`, which was necessary to indicate the length of the borrow of `World`
//   for the function is not the `'w` needed to store the function in `AmethystApplication`.
//   In addition, it requires the `World` (and hence the `ApplicationBuilder`) to be instantiated
//   in a scope greater than the `AmethystApplication`'s lifetime, which detracts from the
//   ergonomics of this test harness.
type FnResourceAdd = Box<dyn FnMut(&mut World) + Send>;
type FnSetup = Box<dyn FnOnce(&mut World) + Send>;
type FnState<T, E> = Box<dyn FnOnce() -> Box<dyn State<T, E>>>;

/// Screen width used in predefined display configuration.
pub const SCREEN_WIDTH: u32 = 800;
/// Screen height used in predefined display configuration.
pub const SCREEN_HEIGHT: u32 = 600;
/// The ratio between the backing framebuffer resolution and the window size in screen pixels.
/// This is typically one for a normal display and two for a retina display.
pub const HIDPI: f64 = 1.;

// Use a mutex to prevent multiple tests that use Rendy from running simultaneously:
//
// <https://github.com/amethyst/rendy/issues/151>
lazy_static! {
    static ref RENDY_MEMORY_MUTEX: Mutex<()> = Mutex::new(());
}

/// Builder for an Amethyst application.
///
/// This provides varying levels of setup so that users do not have to register common bundles.
///
/// # Type Parameters
///
/// * `T`: Game data type that holds the common dispatcher.
/// * `E`: Custom event type shared between states.
#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct AmethystApplication<T, E, R>
where
    E: Send + Sync + 'static,
{
    /// Functions to add bundles to the game data.
    ///
    /// This is necessary because `System`s are not `Send`, and so we cannot send `GameDataBuilder`
    /// across a thread boundary, necessary to run the `Application` in a sub thread to avoid a
    /// segfault caused by mesa and the software GL renderer.
    #[derivative(Debug = "ignore")]
    bundle_add_fns: Vec<BundleAddFn>,
    /// Functions to add bundles to the game data.
    ///
    /// This is necessary because `System`s are not `Send`, and so we cannot send `GameDataBuilder`
    /// across a thread boundary, necessary to run the `Application` in a sub thread to avoid a
    /// segfault caused by mesa and the software GL renderer.
    #[derivative(Debug = "ignore")]
    resource_add_fns: Vec<FnResourceAdd>,
    /// Setup functions to run, in user specified order.
    #[derivative(Debug = "ignore")]
    setup_fns: Vec<FnSetup>,
    /// States to run, in user specified order.
    #[derivative(Debug = "ignore")]
    state_fns: Vec<FnState<T, E>>,
    /// Game data and event type.
    state_data: PhantomData<(T, E, R)>,
}

impl AmethystApplication<GameData<'static, 'static>, StateEvent, StateEventReader> {
    /// Returns an Amethyst application without any bundles.
    pub fn blank() -> AmethystApplication<GameData<'static, 'static>, StateEvent, StateEventReader>
    {
        AmethystApplication {
            bundle_add_fns: Vec::new(),
            resource_add_fns: Vec::new(),
            setup_fns: Vec::new(),
            state_fns: Vec::new(),
            state_data: PhantomData,
        }
    }

    /// Returns an application with the Transform, Input, and UI bundles.
    ///
    /// This also adds a `ScreenDimensions` resource to the `World` so that UI calculations can be
    /// done.
    pub fn ui_base<B: BindingTypes>(
    ) -> AmethystApplication<GameData<'static, 'static>, StateEvent, StateEventReader> {
        AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_ui_bundles::<B>()
            .with_resource(ScreenDimensions::new(SCREEN_WIDTH, SCREEN_HEIGHT, HIDPI))
    }

    /// Returns a `PathBuf` to `<crate_dir>/assets`.
    pub fn assets_dir() -> Result<PathBuf, Error> {
        Ok(application_root_dir()?.join("assets"))
    }
}

impl<E, R> AmethystApplication<GameData<'static, 'static>, E, R>
where
    E: Clone + Send + Sync + 'static,
    R: Default,
{
    /// Returns the built Application.
    ///
    /// If you are intending to run the `Application`, you can use the `run()` or `run_isolated()`
    /// methods instead.
    pub fn build(self) -> Result<CoreApplication<'static, GameData<'static, 'static>, E, R>, Error>
    where
        for<'b> R: EventReader<'b, Event = E>,
    {
        let params = (
            self.bundle_add_fns,
            self.resource_add_fns,
            self.setup_fns,
            self.state_fns,
        );
        Self::build_internal(params)
    }

    // Hack to get around `S` or `T` not being `Send`
    // We take a function that constructs `S`, and the function itself is `Send`.
    // However, `Self` has `PhantomData<T>`, which means we cannot send `self` to a thread. Instead
    // we have to take all of the other fields and send those through.
    //
    // Need to `#[allow(clippy::type_complexity)]` because the type declaration would have unused type
    // parameters which causes a compilation failure.
    #[allow(unknown_lints, clippy::type_complexity)]
    fn build_internal(
        (bundle_add_fns, resource_add_fns, setup_fns, state_fns): (
            Vec<BundleAddFn>,
            Vec<FnResourceAdd>,
            Vec<FnSetup>,
            Vec<FnState<GameData<'static, 'static>, E>>,
        ),
    ) -> Result<CoreApplication<'static, GameData<'static, 'static>, E, R>, Error>
    where
        for<'b> R: EventReader<'b, Event = E>,
    {
        let game_data = bundle_add_fns.into_iter().fold(
            Ok(GameDataBuilder::default()),
            |game_data: Result<GameDataBuilder<'_, '_>, Error>, function: BundleAddFn| {
                game_data.and_then(function)
            },
        )?;

        let mut states = Vec::<Box<dyn State<GameData<'static, 'static>, E>>>::new();
        state_fns
            .into_iter()
            .rev()
            .for_each(|state_fn| states.push(state_fn()));
        Self::build_application(
            SequencerState::new(states),
            game_data,
            resource_add_fns,
            setup_fns,
        )
    }

    fn build_application<S>(
        first_state: S,
        game_data: GameDataBuilder<'static, 'static>,
        resource_add_fns: Vec<FnResourceAdd>,
        setup_fns: Vec<FnSetup>,
    ) -> Result<CoreApplication<'static, GameData<'static, 'static>, E, R>, Error>
    where
        S: State<GameData<'static, 'static>, E> + 'static,
        for<'b> R: EventReader<'b, Event = E>,
    {
        let assets_dir =
            AmethystApplication::assets_dir().expect("Failed to get default assets dir.");
        let mut application_builder = CoreApplication::build(assets_dir, first_state)?;
        {
            let world = &mut application_builder.world;
            for mut function in resource_add_fns {
                function(world);
            }
            for function in setup_fns {
                function(world);
            }
        }
        application_builder.build(game_data)
    }

    /// Runs the application and returns `Ok(())` if nothing went wrong.
    pub fn run(self) -> Result<(), Error>
    where
        for<'b> R: EventReader<'b, Event = E>,
    {
        let params = (
            self.bundle_add_fns,
            self.resource_add_fns,
            self.setup_fns,
            self.state_fns,
        );

        // `CoreApplication` is `!UnwindSafe`, but wrapping it in a `Mutex` allows us to
        // recover from a panic.
        let application = Mutex::new(Self::build_internal(params)?);
        panic::catch_unwind(move || {
            application
                .lock()
                .expect("Expected to get application lock")
                .run()
        })
        .map_err(Self::box_any_to_error)
    }

    /// Run the application in a sub thread.
    ///
    /// Historically this has been used for the following reasons:
    ///
    /// * To avoid segmentation faults using [X and mesa][mesa].
    /// * To avoid multiple threads sharing the same memory in [Vulkan][vulkan].
    ///
    /// This must **NOT** be used when including the `AudioBundle` on Windows, as it causes a
    /// [segfault][audio].
    ///
    /// [mesa]: <https://github.com/rust-windowing/glutin/issues/1038>
    /// [vulkan]: <https://github.com/amethyst/rendy/issues/151>
    /// [audio]: <https://github.com/amethyst/amethyst/issues/1595>
    pub fn run_isolated(self) -> Result<(), Error>
    where
        for<'b> R: EventReader<'b, Event = E>,
    {
        // Acquire a lock due to memory access issues when using Rendy:
        //
        // See: <https://github.com/amethyst/rendy/issues/151>
        let _guard = RENDY_MEMORY_MUTEX.lock().unwrap();

        self.run()
    }

    fn box_any_to_error(error: Box<dyn Any + Send>) -> Error {
        // Caught `panic!`s are generally `&str`s.
        //
        // If we get something else, we just inform the user to check the test output.
        if let Some(inner) = error.downcast_ref::<&str>() {
            Error::from_string(inner.to_string())
        } else {
            Error::from_string(
                "Unable to detect additional information from test failure.\n\
                 Please inspect the test output for clues.",
            )
        }
    }
}

impl<T, E, R> AmethystApplication<T, E, R>
where
    T: GameUpdate + 'static,
    E: Send + Sync + 'static,
    R: 'static,
{
    /// Use the specified custom event type instead of `()`.
    ///
    /// This **must** be invoked before any of the `.with_*()` function calls as the custom event
    /// type parameter is changed, so we are unable to bring any of the existing parameters across.
    ///
    /// # Type Parameters
    ///
    /// * `Evt`: Type used for state events.
    /// * `Rdr`: Event reader of the state events.
    pub fn with_custom_event_type<Evt, Rdr>(self) -> AmethystApplication<T, Evt, Rdr>
    where
        Evt: Send + Sync + 'static,
        for<'b> Rdr: EventReader<'b, Event = Evt>,
    {
        if !self.state_fns.is_empty() {
            panic!(
                "`{}` must be invoked **before** any other `.with_*()` \
                 functions calls.",
                stringify!(with_custom_event_type::<E>())
            );
        }
        AmethystApplication {
            bundle_add_fns: self.bundle_add_fns,
            resource_add_fns: self.resource_add_fns,
            setup_fns: self.setup_fns,
            state_fns: Vec::new(),
            state_data: PhantomData,
        }
    }

    /// Adds a bundle to the list of bundles.
    ///
    /// # Parameters
    ///
    /// * `bundle`: Bundle to add.
    pub fn with_bundle<B>(mut self, bundle: B) -> Self
    where
        B: SystemBundle<'static, 'static> + Send + 'static,
    {
        // We need to use `SendBoxFnOnce` because:
        //
        // * `FnOnce` takes itself by value when you call it.
        // * To pass a `FnOnce` around (transferring ownership), it must be boxed, since it's not
        //   `Sized`.
        // * A `Box<FnOnce()>` is a `Sized` type with a reference to the `FnOnce`
        // * To call the function inside the `Box<FnOnce()>`, it must be moved out of the box
        //   because we need to own the `FnOnce` to be able to call it by value, whereas the `Box`
        //   only holds the reference.
        // * To own it, we would have to move it onto the stack.
        // * However, since it's not `Sized`, we can't do that.
        //
        // To make this work, we can implement a trait for `FnOnce` with a trait function which
        // takes `Box<Self>` and can invoke the `FnOnce` whilst inside the Box.
        // `SendBoxFnOnce` is an implementation of this.
        //
        // See <https://users.rust-lang.org/t/move-a-boxed-function-inside-a-closure/18199>
        self.bundle_add_fns
            .push(Box::new(|game_data: GameDataBuilder<'static, 'static>| {
                game_data.with_bundle(bundle)
            }));
        self
    }

    /// Adds a bundle to the list of bundles.
    ///
    /// This provides an alternative to `.with_bundle(B)` where `B` is `!Send`. The function that
    /// instantiates the bundle must be `Send`.
    ///
    /// # Parameters
    ///
    /// * `bundle_function`: Function to instantiate the Bundle.
    pub fn with_bundle_fn<FnBundle, B>(mut self, bundle_function: FnBundle) -> Self
    where
        FnBundle: FnOnce() -> B + Send + 'static,
        B: SystemBundle<'static, 'static> + 'static,
    {
        self.bundle_add_fns.push(Box::new(
            move |game_data: GameDataBuilder<'static, 'static>| {
                game_data.with_bundle(bundle_function())
            },
        ));
        self
    }

    /// Registers `InputBundle` and `UiBundle` with this application.
    ///
    /// This method is provided to avoid [stringly-typed][stringly] parameters for the Input and UI
    /// bundles. We recommended that you use strong types instead of `<StringBindings>`.
    ///
    /// # Type Parameters
    ///
    /// * `B`: Type representing the input binding types.
    pub fn with_ui_bundles<B: BindingTypes>(self) -> Self {
        self.with_bundle(InputBundle::<B>::new())
            .with_bundle(UiBundle::<B>::new())
    }

    /// Adds a resource to the `World`.
    ///
    /// # Parameters
    ///
    /// * `resource`: Bundle to add.
    pub fn with_resource<Res>(mut self, resource: Res) -> Self
    where
        Res: Resource,
    {
        let mut resource_opt = Some(resource);
        self.resource_add_fns
            .push(Box::new(move |world: &mut World| {
                let resource = resource_opt.take();
                if let Some(resource) = resource {
                    world.insert(resource);
                }
            }));
        self
    }

    /// Adds a state to run in the Amethyst application.
    ///
    /// # Parameters
    ///
    /// * `state_fn`: `State` to use.
    pub fn with_state<S, FnStateLocal>(mut self, state_fn: FnStateLocal) -> Self
    where
        S: State<T, E> + 'static,
        FnStateLocal: FnOnce() -> S + Send + Sync + 'static,
    {
        // Box up the state
        let closure = move || Box::new((state_fn)()) as Box<dyn State<T, E>>;
        self.state_fns.push(Box::new(closure));
        self
    }

    /// Registers a `System` into this application's `GameData`.
    ///
    /// # Parameters
    ///
    /// * `system`: `System` to run.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with_system<S, N>(self, system: S, name: N, deps: &[N]) -> Self
    where
        S: for<'sys_local> System<'sys_local> + Send + 'static,
        N: Into<String> + Clone,
    {
        let name = Into::<String>::into(name);
        let deps = deps
            .iter()
            .map(Clone::clone)
            .map(Into::<String>::into)
            .collect::<Vec<String>>();
        self.with_bundle_fn(move || SystemInjectionBundle::new(system, name, deps))
    }

    /// Registers a `System` into this application's `GameData`.
    ///
    /// # Parameters
    ///
    /// * `system_desc`: Descriptor to instantiate the `System`.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with_system_desc<SD, S, N>(self, system_desc: SD, name: N, deps: &[N]) -> Self
    where
        SD: SystemDesc<'static, 'static, S> + Send + Sync + 'static,
        S: for<'sys_local> System<'sys_local> + Send + 'static,
        N: Into<String> + Clone,
    {
        let name = Into::<String>::into(name);
        let deps = deps
            .iter()
            .map(Clone::clone)
            .map(Into::<String>::into)
            .collect::<Vec<String>>();
        self.with_bundle_fn(move || SystemDescInjectionBundle::new(system_desc, name, deps))
    }

    /// Registers a thread local `System` into this application's `GameData`.
    ///
    /// # Parameters
    ///
    /// * `run_now_desc`: Descriptor to instantiate the thread local system.
    pub fn with_thread_local<RNDesc, RN>(self, run_now_desc: RNDesc) -> Self
    where
        RNDesc: RunNowDesc<'static, 'static, RN> + Send + Sync + 'static,
        RN: for<'sys_local> RunNow<'sys_local> + Send + 'static,
    {
        self.with_bundle_fn(move || ThreadLocalInjectionBundle::new(run_now_desc))
    }

    /// Registers a `System` to run in a `CustomDispatcherState`.
    ///
    /// This will run the system once in a dedicated `State`, allowing you to inspect the effects of
    /// the system after setting up the world to a desired state.
    ///
    /// # Parameters
    ///
    /// * `system`: `System` to run.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with_system_single<S, N>(self, system: S, name: N, deps: &[N]) -> Self
    where
        S: for<'sys_local> System<'sys_local> + Send + Sync + 'static,
        N: Into<String> + Clone,
    {
        let name = Into::<String>::into(name);
        let deps = deps
            .iter()
            .map(Clone::clone)
            .map(Into::<String>::into)
            .collect::<Vec<String>>();
        self.with_state(move || {
            CustomDispatcherStateBuilder::new()
                .with_system(system, name, deps)
                .build()
        })
    }

    /// Registers a `System` to run in a `CustomDispatcherState`.
    ///
    /// This will run the system once in a dedicated `State`, allowing you to inspect the effects of
    /// the system after setting up the world to a desired state.
    ///
    /// # Parameters
    ///
    /// * `system_desc`: Descriptor to instantiate the `System`.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with_system_desc_single<SD, S, N>(self, system_desc: SD, name: N, deps: &[N]) -> Self
    where
        SD: SystemDesc<'static, 'static, S> + Send + Sync + 'static,
        S: for<'sys_local> System<'sys_local> + Send + Sync + 'static,
        N: Into<String> + Clone,
    {
        let name = Into::<String>::into(name);
        let deps = deps
            .iter()
            .map(Clone::clone)
            .map(Into::<String>::into)
            .collect::<Vec<String>>();
        self.with_state(move || {
            CustomDispatcherStateBuilder::new()
                .with_system_desc(system_desc, name, deps)
                .build()
        })
    }

    /// Registers a function to run in the `World`.
    ///
    /// # Parameters
    ///
    /// * `func`: Function to execute.
    pub fn with_fn<F>(self, func: F) -> Self
    where
        F: Fn(&mut World) + Send + Sync + 'static,
    {
        self.with_state(move || FunctionState::new(func))
    }

    /// Registers a function that sets up the `World`.
    ///
    /// This is an alias to `.with_fn(F)`.
    ///
    /// # Parameters
    ///
    /// * `setup_fn`: Function to execute.
    pub fn with_setup<F>(mut self, setup_fn: F) -> Self
    where
        F: FnOnce(&mut World) + Send + 'static,
    {
        self.setup_fns.push(Box::new(setup_fn));
        self
    }

    /// Registers a function that executes a desired effect.
    ///
    /// This is an alias to `.with_fn(F)`.
    ///
    /// # Parameters
    ///
    /// * `effect_fn`: Function that executes an effect.
    pub fn with_effect<F>(self, effect_fn: F) -> Self
    where
        F: Fn(&mut World) + Send + Sync + 'static,
    {
        self.with_fn(effect_fn)
    }

    /// Registers a function to assert an expected outcome.
    ///
    /// This is an alias to `.with_fn(F)`.
    ///
    /// # Parameters
    ///
    /// * `assertion_fn`: Function that asserts the expected state.
    pub fn with_assertion<F>(self, assertion_fn: F) -> Self
    where
        F: Fn(&mut World) + Send + Sync + 'static,
    {
        self.with_fn(assertion_fn)
    }
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use amethyst::{
        assets::{Asset, AssetStorage, Handle, Loader, ProcessingState, Processor},
        core::{bundle::SystemBundle, SystemDesc},
        derive::SystemDesc,
        ecs::prelude::*,
        error::Error,
        prelude::*,
        ui::FontAsset,
        window::ScreenDimensions,
    };

    use super::AmethystApplication;
    use crate::{EffectReturn, FunctionState, PopState};

    #[test]
    fn bundle_build_is_ok() -> Result<(), Error> {
        AmethystApplication::blank().with_bundle(BundleZero).run()
    }

    #[test]
    fn load_multiple_bundles() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_bundle(BundleZero)
            .with_bundle(BundleOne)
            .run()
    }

    #[test]
    fn assertion_when_resource_is_added_succeeds() -> Result<(), Error> {
        let assertion_fn = |world: &mut World| {
            world.read_resource::<ApplicationResource>();
            world.read_resource::<ApplicationResourceNonDefault>();
        };

        AmethystApplication::blank()
            .with_bundle(BundleZero)
            .with_bundle(BundleOne)
            .with_assertion(assertion_fn)
            .run()
    }

    #[test]
    #[should_panic] // This cannot be expect explicit because of nightly feature.
    fn assertion_when_resource_is_not_added_should_panic() {
        let assertion_fn = |world: &mut World| {
            // Panics if `ApplicationResource` was not added.
            world.read_resource::<ApplicationResource>();
        };

        AmethystApplication::blank()
            // without BundleOne
            .with_assertion(assertion_fn)
            .run()
            .unwrap();
    }

    #[test]
    fn assertion_switch_with_loading_state_with_add_resource_succeeds() -> Result<(), Error> {
        let state_fns = || {
            let assertion_fn = |world: &mut World| {
                world.read_resource::<LoadResource>();
            };

            // Necessary if the State being tested is a loading state that returns `Trans::Switch`
            let assertion_state = FunctionState::new(assertion_fn);
            LoadingState::new(assertion_state)
        };

        AmethystApplication::blank().with_state(state_fns).run()
    }

    #[test]
    fn assertion_push_with_loading_state_with_add_resource_succeeds() -> Result<(), Error> {
        // Alternative to embedding the `FunctionState` is to switch to a `PopState` but still
        // provide the assertion function
        let state_fns = || LoadingState::new(PopState);
        let assertion_fn = |world: &mut World| {
            world.read_resource::<LoadResource>();
        };

        AmethystApplication::blank()
            .with_state(state_fns)
            .with_assertion(assertion_fn)
            .run()
    }

    #[test]
    #[should_panic] // This cannot be expect explicit because of nightly feature.
    fn assertion_switch_with_loading_state_without_add_resource_should_panic() {
        let state_fns = || {
            let assertion_fn = |world: &mut World| {
                world.read_resource::<LoadResource>();
            };

            SwitchState::new(FunctionState::new(assertion_fn))
        };

        AmethystApplication::blank()
            .with_state(state_fns)
            .run()
            .unwrap();
    }

    #[test]
    #[should_panic] // This cannot be expect explicit because of nightly feature.
    fn assertion_push_with_loading_state_without_add_resource_should_panic() {
        // Alternative to embedding the `FunctionState` is to switch to a `PopState` but still
        // provide the assertion function
        let state_fns = || SwitchState::new(PopState);
        let assertion_fn = |world: &mut World| {
            world.read_resource::<LoadResource>();
        };

        AmethystApplication::blank()
            .with_state(state_fns)
            .with_assertion(assertion_fn)
            .run()
            .unwrap();
    }

    #[test]
    fn game_data_must_update_before_assertion() -> Result<(), Error> {
        let effect_fn = |world: &mut World| {
            let handles = vec![
                AssetZeroLoader::load(world, AssetZero(10)).unwrap(),
                AssetZeroLoader::load(world, AssetZero(20)).unwrap(),
            ];

            world.insert::<Vec<AssetZeroHandle>>(handles);
        };
        let assertion_fn = |world: &mut World| {
            let asset_translation_zero_handles = world.read_resource::<Vec<AssetZeroHandle>>();

            let store = world.read_resource::<AssetStorage<AssetZero>>();
            assert_eq!(
                Some(&AssetZero(10)),
                store.get(&asset_translation_zero_handles[0])
            );
            assert_eq!(
                Some(&AssetZero(20)),
                store.get(&asset_translation_zero_handles[1])
            );
        };

        AmethystApplication::blank()
            .with_bundle(BundleAsset)
            .with_effect(effect_fn)
            .with_assertion(assertion_fn)
            .run()
    }

    #[test]
    fn execution_order_is_setup_state_effect_assertion() -> Result<(), Error> {
        struct Setup;
        let setup_fns = |world: &mut World| world.insert(Setup);
        let state_fns = || {
            LoadingState::new(FunctionState::new(|world: &mut World| {
                // Panics if setup is not run before this.
                world.read_resource::<Setup>();
            }))
        };
        let effect_fn = |world: &mut World| {
            // If `LoadingState` is not run before this, this will panic
            world.read_resource::<LoadResource>();

            let handles = vec![AssetZeroLoader::load(world, AssetZero(10)).unwrap()];
            world.insert(handles);
        };
        let assertion_fn = |world: &mut World| {
            let asset_translation_zero_handles = world.read_resource::<Vec<AssetZeroHandle>>();

            let store = world.read_resource::<AssetStorage<AssetZero>>();
            assert_eq!(
                Some(&AssetZero(10)),
                store.get(&asset_translation_zero_handles[0])
            );
        };

        AmethystApplication::blank()
            .with_bundle(BundleAsset)
            .with_setup(setup_fns)
            .with_state(state_fns)
            .with_effect(effect_fn)
            .with_assertion(assertion_fn)
            .run()
    }

    #[test]
    fn base_application_can_load_ui() -> Result<(), Error> {
        let assertion_fn = |world: &mut World| {
            // Next line would panic if `UiBundle` wasn't added.
            world.read_resource::<AssetStorage<FontAsset>>();
            // `.base()` should add `ScreenDimensions` as this is necessary for `UiBundle` to
            // initialize properly.
            world.read_resource::<ScreenDimensions>();
        };

        AmethystApplication::ui_base::<amethyst::input::StringBindings>()
            .with_assertion(assertion_fn)
            .run()
    }

    #[test]
    fn with_system_runs_system_every_tick() -> Result<(), Error> {
        let effect_fn = |world: &mut World| {
            let entity = world.create_entity().with(ComponentZero(0)).build();

            world.insert(EffectReturn(entity));
        };

        fn get_component_zero_value(world: &mut World) -> i32 {
            let entity = world.read_resource::<EffectReturn<Entity>>().0;

            let component_zero_storage = world.read_storage::<ComponentZero>();
            let component_zero = component_zero_storage
                .get(entity)
                .expect("Entity should have a `ComponentZero` component.");

            component_zero.0
        };

        AmethystApplication::blank()
            .with_system(SystemEffect, "system_effect", &[])
            .with_effect(effect_fn)
            .with_assertion(|world| assert_eq!(1, get_component_zero_value(world)))
            .with_assertion(|world| assert_eq!(2, get_component_zero_value(world)))
            .run()
    }

    #[test]
    fn with_system_invoked_twice_should_not_panic() {
        AmethystApplication::blank()
            .with_system(SystemZero, "zero", &[])
            .with_system(SystemOne, "one", &["zero"]);
    }

    #[test]
    fn with_system_single_runs_system_once() -> Result<(), Error> {
        let assertion_fn = |world: &mut World| {
            let entity = world.read_resource::<EffectReturn<Entity>>().0;

            let component_zero_storage = world.read_storage::<ComponentZero>();
            let component_zero = component_zero_storage
                .get(entity)
                .expect("Entity should have a `ComponentZero` component.");

            // If the system ran, the value in the `ComponentZero` should be 1.
            assert_eq!(1, component_zero.0);
        };

        AmethystApplication::blank()
            .with_setup(|world| {
                world.register::<ComponentZero>();

                let entity = world.create_entity().with(ComponentZero(0)).build();
                world.insert(EffectReturn(entity));
            })
            .with_system_single(SystemEffect, "system_effect", &[])
            .with_assertion(assertion_fn)
            .with_assertion(assertion_fn)
            .run()
    }

    // Double usage tests
    // If the second call panics, then the setup functions were not executed in the right order.

    #[test]
    fn with_setup_invoked_twice_should_run_in_specified_order() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_setup(|world| {
                world.insert(ApplicationResource);
            })
            .with_setup(|world| {
                world.read_resource::<ApplicationResource>();
            })
            .run()
    }

    #[test]
    fn with_effect_invoked_twice_should_run_in_the_specified_order() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_effect(|world| {
                world.insert(ApplicationResource);
            })
            .with_effect(|world| {
                world.read_resource::<ApplicationResource>();
            })
            .run()
    }

    #[test]
    fn with_assertion_invoked_twice_should_run_in_the_specified_order() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_assertion(|world| {
                world.insert(ApplicationResource);
            })
            .with_assertion(|world| {
                world.read_resource::<ApplicationResource>();
            })
            .run()
    }

    #[test]
    fn with_state_invoked_twice_should_run_in_the_specified_order() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_state(|| {
                FunctionState::new(|world| {
                    world.insert(ApplicationResource);
                })
            })
            .with_state(|| {
                FunctionState::new(|world| {
                    world.read_resource::<ApplicationResource>();
                })
            })
            .run()
    }

    #[test]
    fn with_state_invoked_after_with_resource_should_work() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_resource(ApplicationResource)
            .with_state(|| {
                FunctionState::new(|world| {
                    world.read_resource::<ApplicationResource>();
                })
            })
            .run()
    }

    #[test]
    fn setup_runs_before_system() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_setup(|world| world.insert(ApplicationResourceNonDefault))
            .with_system(SystemNonDefault, "", &[])
            .run()
    }

    /// This is here because on Windows, a segmentation fault happens when:
    ///
    /// * There are multiple threads, each with its own sub-thread in the same application.
    /// * One of the sub-threads initializes a COM object in a `lazy_static` variable.
    /// * That sub-thread is joined.
    /// * Another sub-thread accesses the COM object through the same `lazy_static` variable.
    ///
    /// To check this:
    ///
    /// ```bash
    /// # On Windows
    /// cd amethyst_test
    /// cargo test --features audio
    /// ```
    ///
    /// **Note:** If you simply need an audio file to be loaded, just add a `Processor::<Source>` in
    /// the test setup:
    ///
    /// ```rust,ignore
    /// use amethyst::{assets::Processor, audio::Source};
    ///
    /// AmethystApplication::blank()
    ///     .with_system(Processor::<Source>::new(), "source_processor", &[])
    ///     // ...
    /// ```
    ///
    /// For more details, see <https://github.com/amethyst/amethyst/issues/1595>.
    #[cfg(feature = "audio")]
    mod audio_test {
        use amethyst::{
            assets::AssetStorage,
            audio::{AudioBundle, Source},
            error::Error,
        };

        use super::AmethystApplication;

        #[test]
        fn audio_zero() -> Result<(), Error> {
            AmethystApplication::blank()
                .with_bundle(AudioBundle::default())
                .with_assertion(|world| {
                    world.read_resource::<AssetStorage<Source>>();
                })
                .run()
        }

        #[test]
        fn audio_one() -> Result<(), Error> {
            AmethystApplication::blank()
                .with_bundle(AudioBundle::default())
                .with_assertion(|world| {
                    world.read_resource::<AssetStorage<Source>>();
                })
                .run()
        }

        #[test]
        fn audio_two() -> Result<(), Error> {
            AmethystApplication::blank()
                .with_bundle_fn(|| AudioBundle::default())
                .with_assertion(|world| {
                    world.read_resource::<AssetStorage<Source>>();
                })
                .run()
        }

        #[test]
        fn audio_three() -> Result<(), Error> {
            AmethystApplication::blank()
                .with_bundle_fn(|| AudioBundle::default())
                .with_assertion(|world| {
                    world.read_resource::<AssetStorage<Source>>();
                })
                .run()
        }
    }

    // === Resources === //
    #[derive(Debug, Default)]
    struct ApplicationResource;
    #[derive(Debug)]
    struct ApplicationResourceNonDefault;
    #[derive(Debug)]
    struct LoadResource;

    // === States === //
    struct LoadingState<'a, 'b, S, E>
    where
        S: State<GameData<'a, 'b>, E> + 'static,
        E: Send + Sync + 'static,
    {
        next_state: Option<S>,
        state_data: PhantomData<dyn State<GameData<'a, 'b>, E>>,
    }
    impl<'a, 'b, S, E> LoadingState<'a, 'b, S, E>
    where
        S: State<GameData<'a, 'b>, E> + 'static,
        E: Send + Sync + 'static,
    {
        fn new(next_state: S) -> Self {
            LoadingState {
                next_state: Some(next_state),
                state_data: PhantomData,
            }
        }
    }
    impl<'a, 'b, S, E> State<GameData<'a, 'b>, E> for LoadingState<'a, 'b, S, E>
    where
        S: State<GameData<'a, 'b>, E> + 'static,
        E: Send + Sync + 'static,
    {
        fn update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> Trans<GameData<'a, 'b>, E> {
            data.data.update(&data.world);
            data.world.insert(LoadResource);
            Trans::Switch(Box::new(self.next_state.take().unwrap()))
        }
    }

    struct SwitchState<S, T, E>
    where
        S: State<T, E>,
        E: Send + Sync + 'static,
    {
        next_state: Option<S>,
        state_data: PhantomData<(T, E)>,
    }
    impl<S, T, E> SwitchState<S, T, E>
    where
        S: State<T, E>,
        E: Send + Sync + 'static,
    {
        fn new(next_state: S) -> Self {
            SwitchState {
                next_state: Some(next_state),
                state_data: PhantomData,
            }
        }
    }
    impl<S, T, E> State<T, E> for SwitchState<S, T, E>
    where
        S: State<T, E> + 'static,
        E: Send + Sync + 'static,
    {
        fn update(&mut self, _data: StateData<'_, T>) -> Trans<T, E> {
            Trans::Switch(Box::new(self.next_state.take().unwrap()))
        }
    }

    // === Systems === //
    #[derive(Debug, SystemDesc)]
    struct SystemZero;
    impl<'s> System<'s> for SystemZero {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    #[derive(Debug, SystemDesc)]
    struct SystemOne;
    type SystemOneData<'s> = Read<'s, ApplicationResource>;
    impl<'s> System<'s> for SystemOne {
        type SystemData = SystemOneData<'s>;
        fn run(&mut self, _: Self::SystemData) {}
    }

    #[derive(Debug, SystemDesc)]
    #[system_desc(insert(ApplicationResourceNonDefault))]
    struct SystemNonDefault;
    type SystemNonDefaultData<'s> = ReadExpect<'s, ApplicationResourceNonDefault>;
    impl<'s> System<'s> for SystemNonDefault {
        type SystemData = SystemNonDefaultData<'s>;
        fn run(&mut self, _: Self::SystemData) {}
    }

    #[derive(Debug, SystemDesc)]
    struct SystemEffect;
    type SystemEffectData<'s> = WriteStorage<'s, ComponentZero>;
    impl<'s> System<'s> for SystemEffect {
        type SystemData = SystemEffectData<'s>;
        fn run(&mut self, mut component_zero_storage: Self::SystemData) {
            for mut component_zero in (&mut component_zero_storage).join() {
                component_zero.0 += 1
            }
        }
    }

    // === Bundles === //
    #[derive(Debug)]
    struct BundleZero;
    impl<'a, 'b> SystemBundle<'a, 'b> for BundleZero {
        fn build(
            self,
            _world: &mut World,
            builder: &mut DispatcherBuilder<'a, 'b>,
        ) -> Result<(), Error> {
            builder.add(SystemZero, "system_zero", &[]);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct BundleOne;
    impl<'a, 'b> SystemBundle<'a, 'b> for BundleOne {
        fn build(
            self,
            world: &mut World,
            builder: &mut DispatcherBuilder<'a, 'b>,
        ) -> Result<(), Error> {
            builder.add(SystemOne, "system_one", &["system_zero"]);
            builder.add(SystemNonDefault.build(world), "system_non_default", &[]);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct BundleAsset;
    impl<'a, 'b> SystemBundle<'a, 'b> for BundleAsset {
        fn build(
            self,
            _world: &mut World,
            builder: &mut DispatcherBuilder<'a, 'b>,
        ) -> Result<(), Error> {
            builder.add(
                Processor::<AssetZero>::new(),
                "asset_translation_zero_processor",
                &[],
            );
            Ok(())
        }
    }

    // === Assets === //
    #[derive(Debug, PartialEq)]
    struct AssetZero(u32);
    impl Asset for AssetZero {
        const NAME: &'static str = "amethyst_test::AssetZero";
        type Data = Self;
        type HandleStorage = VecStorage<Handle<Self>>;
    }
    impl Component for AssetZero {
        type Storage = DenseVecStorage<Self>;
    }
    impl From<AssetZero> for Result<ProcessingState<AssetZero>, Error> {
        fn from(asset_translation_zero: AssetZero) -> Result<ProcessingState<AssetZero>, Error> {
            Ok(ProcessingState::Loaded(asset_translation_zero))
        }
    }
    type AssetZeroHandle = Handle<AssetZero>;

    // === System delegates === //
    struct AssetZeroLoader;
    impl AssetZeroLoader {
        fn load(
            world: &World,
            asset_translation_zero: AssetZero,
        ) -> Result<AssetZeroHandle, Error> {
            let loader = world.read_resource::<Loader>();
            Ok(loader.load_from_data(
                asset_translation_zero,
                (),
                &world.read_resource::<AssetStorage<AssetZero>>(),
            ))
        }
    }

    // === Components === //
    struct ComponentZero(pub i32);
    impl Component for ComponentZero {
        type Storage = DenseVecStorage<Self>;
    }
}

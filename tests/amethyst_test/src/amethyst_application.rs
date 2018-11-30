use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::thread;

use amethyst::{
    self,
    animation::AnimationBundle,
    core::{transform::TransformBundle, EventReader, SystemBundle},
    ecs::prelude::*,
    input::InputBundle,
    prelude::*,
    renderer::{
        ColorMask, DepthMode, DisplayConfig, DrawFlat2D, Material, Pipeline, PipelineBuilder,
        RenderBundle, ScreenDimensions, SpriteRender, Stage, StageBuilder, ALPHA,
    },
    shred::Resource,
    ui::{DrawUi, UiBundle},
    utils::application_root_dir,
    Result, StateEventReader,
};
use boxfnonce::SendBoxFnOnce;
use hetseq::Queue;
use std::collections::VecDeque;

use crate::{
    state::sequencer::{Step, WorldFn},
    CustomDispatcherStateBuilder, Sequencer, SystemInjectionBundle,
};

type BundleAddFn = SendBoxFnOnce<
    'static,
    (GameDataBuilder<'static, 'static>,),
    Result<GameDataBuilder<'static, 'static>>,
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
type FnResourceAdd = Box<FnMut(&mut World) + Send>;
/// Constructs a registered state.
type FnStateAdd<S, E> = SendBoxFnOnce<'static, (), Box<StateCallback<S, E>>>;

type DefaultPipeline = PipelineBuilder<
    Queue<(
        Queue<()>,
        StageBuilder<Queue<(Queue<(Queue<()>, DrawFlat2D)>, DrawUi)>>,
    )>,
>;

/// Screen width used in predefined display configuration.
pub const SCREEN_WIDTH: u32 = 800;
/// Screen height used in predefined display configuration.
pub const SCREEN_HEIGHT: u32 = 600;
/// The ratio between the backing framebuffer resolution and the window size in screen pixels.
/// This is typically one for a normal display and two for a retina display.
pub const HIDPI: f64 = 1.;

// Use a mutex to prevent multiple tests that open GL windows from running simultaneously, due to
// race conditions causing failures in X.
// <https://github.com/tomaka/glutin/issues/1038>
lazy_static! {
    static ref X11_GL_MUTEX: Mutex<()> = Mutex::new(());
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
pub struct AmethystApplication<S, E, R> {
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
    /// States to run, in user specified order.
    #[derivative(Debug = "ignore")]
    states: Vec<(S, FnStateAdd<S, E>)>,
    /// Functions to call, one after another.
    #[derivative(Debug = "ignore")]
    functions: VecDeque<WorldFn<S>>,
    /// Game data and event type.
    state_data: PhantomData<R>,
    /// Whether or not this application uses the `RenderBundle`.
    render: bool,
}

impl<S> AmethystApplication<S, StateEvent, StateEventReader>
where
    S: 'static + Send + Sync + Clone + State<StateEvent>,
{
    /// Returns an Amethyst application without any bundles.
    pub fn blank() -> AmethystApplication<S, StateEvent, StateEventReader> {
        AmethystApplication {
            bundle_add_fns: Vec::new(),
            resource_add_fns: Vec::new(),
            states: Vec::new(),
            functions: VecDeque::new(),
            state_data: PhantomData,
            render: false,
        }
    }

    /// Returns an application with the Transform, Input, and UI bundles.
    ///
    /// This also adds a `ScreenDimensions` resource to the `World` so that UI calculations can be
    /// done.
    pub fn ui_base<AX, AC>() -> AmethystApplication<S, StateEvent, StateEventReader>
    where
        AX: Hash + Eq + Clone + Send + Sync + 'static,
        AC: Hash + Eq + Clone + Send + Sync + 'static,
    {
        AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_ui_bundles::<AX, AC>()
            .with_resource(ScreenDimensions::new(SCREEN_WIDTH, SCREEN_HEIGHT, HIDPI))
    }

    /// Returns an application with the Animation, Transform, and Render bundles.
    ///
    /// If you requite `InputBundle` and `UiBundle`, you can call the `with_ui_bundles::<AX, AC>()`
    /// method.
    ///
    /// # Parameters
    ///
    /// * `test_name`: Name of the test, used to populate the window title.
    /// * `visibility`: Whether the window should be visible.
    ///
    /// [stringly]: http://wiki.c2.com/?StringlyTyped
    pub fn render_base<'name, N>(
        test_name: N,
        visibility: bool,
    ) -> AmethystApplication<S, StateEvent, StateEventReader>
    where
        N: Into<&'name str>,
    {
        AmethystApplication::blank()
            .with_bundle(AnimationBundle::<u32, Material>::new(
                "material_animation_control_system",
                "material_sampler_interpolation_system",
            )).with_bundle(AnimationBundle::<u32, SpriteRender>::new(
                "sprite_render_animation_control_system",
                "sprite_render_sampler_interpolation_system",
            )).with_bundle(TransformBundle::new().with_dep(&[
                "material_animation_control_system",
                "material_sampler_interpolation_system",
                "sprite_render_animation_control_system",
                "sprite_render_sampler_interpolation_system",
            ])).with_render_bundle(test_name, visibility)
    }
}

impl<S, E, R> AmethystApplication<S, E, R> {
    /// Returns a `String` to `<crate_dir>/assets`.
    pub fn assets_dir() -> String {
        format!("{}/assets", application_root_dir())
    }
}

impl<S, E, R> AmethystApplication<S, E, R>
where
    S: 'static + Clone + Default + Send + Sync + State<E>,
    E: 'static + Clone + Send + Sync,
    R: Default,
{
    /// Returns the built Application.
    ///
    /// If you are intending to call `.run()` on the `Application` in a test, be aware that on
    /// Linux, this will cause a segfault when `RenderBundle` is added and GL is using software
    /// rendering, such as when using Xvfb or when the following environmental variable is set:
    /// `LIBGL_ALWAYS_SOFTWARE=1`.
    ///
    /// To avoid this, please call `.run()` instead of this method, which runs the application in a
    /// separate thread and waits for it to end before returning.
    ///
    /// See <https://users.rust-lang.org/t/trouble-identifying-cause-of-segfault/18096>
    pub fn build(self) -> Result<CoreApplication<'static, 'static, S, E, R>>
    where
        for<'b> R: EventReader<'b, Event = E>,
    {
        Self::build_internal(
            self.bundle_add_fns,
            self.resource_add_fns,
            self.states,
            self.functions,
        )
    }

    // Hack to get around `S` or `T` not being `Send`
    // We take a function that constructs `S`, and the function itself is `Send`.
    // However, `Self` has `PhantomData<T>`, which means we cannot send `self` to a thread. Instead
    // we have to take all of the other fields and send those through.
    //
    // Need to `#[allow(type_complexity)]` because the type declaration would have unused type
    // parameters which causes a compilation failure.
    fn build_internal(
        bundle_add_fns: Vec<BundleAddFn>,
        resource_add_fns: Vec<FnResourceAdd>,
        states: Vec<(S, FnStateAdd<S, E>)>,
        functions: VecDeque<WorldFn<S>>,
    ) -> Result<CoreApplication<'static, 'static, S, E, R>>
    where
        for<'b> R: EventReader<'b, Event = E>,
    {
        let game_data = bundle_add_fns.into_iter().fold(
            Ok(GameDataBuilder::default()),
            |game_data: Result<GameDataBuilder>, function: BundleAddFn| {
                game_data.and_then(|game_data| function.call(game_data))
            },
        )?;

        Self::build_application(game_data, resource_add_fns, states, functions)
    }

    fn build_application(
        game_data: GameDataBuilder<'static, 'static>,
        resource_add_fns: Vec<FnResourceAdd>,
        states: Vec<(S, FnStateAdd<S, E>)>,
        functions: VecDeque<WorldFn<S>>,
    ) -> Result<CoreApplication<'static, 'static, S, E, R>>
    where
        for<'b> R: EventReader<'b, Event = E>,
    {
        let mut application_builder = CoreApplication::build(Self::assets_dir())?;

        for (state, callback) in states {
            let callback = callback.call();
            application_builder = application_builder.with_boxed_state(state, callback)?;
        }

        // Calls to perform once.
        application_builder = application_builder.with_global(Sequencer::new(functions));

        {
            let world = &mut application_builder.world;
            for mut function in resource_add_fns {
                function(world);
            }
        }

        application_builder.build(game_data)
    }

    /// Runs the application and returns `Ok(())` if nothing went wrong.
    ///
    /// This method should be called instead of the `.build()` method if the application is to be
    /// run, as this avoids a segfault on Linux when using the GL software renderer.
    pub fn run(self) -> Result<()>
    where
        for<'b> R: EventReader<'b, Event = E>,
    {
        let AmethystApplication {
            bundle_add_fns,
            resource_add_fns,
            states,
            functions,
            render,
            ..
        } = self;

        // Run in a sub thread due to mesa's threading issues with GL software rendering
        // See: <https://users.rust-lang.org/t/trouble-identifying-cause-of-segfault/18096>
        thread::spawn(move || -> Result<()> {
            amethyst::start_logger(Default::default());

            if render {
                let guard = X11_GL_MUTEX.lock().unwrap();

                // Note: if this panics, the Mutex is poisoned.
                // Unfortunately we cannot catch panics, as the application is `!UnwindSafe`
                //
                // We have to build the application after acquiring the lock because the window is
                // already instantiated during the build.
                //
                // The mutex greatly reduces, but does not eliminate X11 window initialization
                // errors from happening:
                //
                // * <https://github.com/tomaka/glutin/issues/1034> can still happen
                // * <https://github.com/tomaka/glutin/issues/1038> may be completely removed
                Self::build_internal(bundle_add_fns, resource_add_fns, states, functions)?.run();

                drop(guard);
            } else {
                Self::build_internal(bundle_add_fns, resource_add_fns, states, functions)?.run();
            }

            Ok(())
        }).join()
        .expect("Failed to run Amethyst application")
    }
}

impl<S, E, R> AmethystApplication<S, E, R>
where
    S: 'static + Clone + Send + Sync + State<E>,
    E: 'static + Send + Sync,
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
    pub fn with_custom_event_type<Evt, Rdr>(self) -> AmethystApplication<S, Evt, Rdr>
    where
        Evt: Send + Sync + 'static,
        for<'b> Rdr: EventReader<'b, Event = Evt>,
    {
        if !self.states.is_empty() {
            panic!(
                "`{}` must be invoked **before** any other `.with_*()` \
                 functions calls.",
                stringify!(with_custom_event_type::<E>())
            );
        }

        AmethystApplication {
            bundle_add_fns: self.bundle_add_fns,
            resource_add_fns: self.resource_add_fns,
            states: Default::default(),
            functions: Default::default(),
            state_data: PhantomData,
            render: self.render,
        }
    }

    /// Adds a bundle to the list of bundles.
    ///
    /// **Note:** If you are adding the `RenderBundle`, you need to use `.with_bundle_fn(F)` as the
    /// `Pipeline` type used by the bundle is `!Send`. Furthermore, you must also invoke
    /// `.mark_render()` to avoid a race condition that causes render tests to fail.
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
        self.bundle_add_fns.push(SendBoxFnOnce::from(
            |game_data: GameDataBuilder<'static, 'static>| game_data.with_bundle(bundle),
        ));
        self
    }

    /// Adds a bundle to the list of bundles.
    ///
    /// This provides an alternative to `.with_bundle(B)` where `B` is `!Send`. The function that
    /// instantiates the bundle must be `Send`.
    ///
    /// **Note:** If you are adding the `RenderBundle`, you must also invoke `.mark_render()` to
    /// avoid a race condition that causes render tests to fail.
    ///
    /// **Note:** There is a `.with_render_bundle()` convenience function if you just need the
    /// `RenderBundle` with predefined parameters.
    ///
    /// # Parameters
    ///
    /// * `bundle_function`: Function to instantiate the Bundle.
    pub fn with_bundle_fn<FnBundle, B>(mut self, bundle_function: FnBundle) -> Self
    where
        FnBundle: FnOnce() -> B + Send + 'static,
        B: SystemBundle<'static, 'static> + 'static,
    {
        self.bundle_add_fns.push(SendBoxFnOnce::from(
            move |game_data: GameDataBuilder<'static, 'static>| {
                game_data.with_bundle(bundle_function())
            },
        ));
        self
    }

    /// Registers `InputBundle` and `UiBundle` with this application.
    ///
    /// This method is provided to avoid [stringly-typed][stringly] parameters for the Input and UI
    /// bundles. We recommended that you use strong types instead of `<String, String>`.
    ///
    /// # Type Parameters
    ///
    /// * `AX`: Type representing the movement axis.
    /// * `AC`: Type representing actions.
    pub fn with_ui_bundles<AX, AC>(self) -> Self
    where
        AX: Hash + Eq + Clone + Send + Sync + 'static,
        AC: Hash + Eq + Clone + Send + Sync + 'static,
    {
        self.with_bundle(InputBundle::<AX, AC>::new())
            .with_bundle(UiBundle::<AX, AC>::new())
    }

    /// Registers the `RenderBundle` with this application.
    ///
    /// This is a convenience function that registers the `RenderBundle` using the predefined
    /// [`display_config`][disp] and [`pipeline`][pipe].
    ///
    /// # Parameters
    ///
    /// * `title`: Window title.
    /// * `visibility`: Whether the window should be visible.
    ///
    /// [disp]: #method.display_config
    /// [pipe]: #method.pipeline
    pub fn with_render_bundle<'name, N>(self, title: N, visibility: bool) -> Self
    where
        N: Into<&'name str>,
    {
        // TODO: We can default to the function name once this RFC is implemented:
        // <https://github.com/rust-lang/rfcs/issues/1743>
        // <https://github.com/rust-lang/rfcs/pull/1719>
        let title = title.into().to_string();

        let display_config = Self::display_config(title, visibility);
        let render_bundle_fn = move || {
            RenderBundle::new(Self::pipeline(), Some(display_config)).with_sprite_sheet_processor()
        };

        self.with_bundle_fn(render_bundle_fn).mark_render()
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
                if resource.is_some() {
                    world.add_resource(resource.unwrap());
                }
            }));
        self
    }

    /// Register a callback associated with a specific state.
    ///
    /// # Arguments
    ///
    /// - `state`: State that callback is associated with.
    /// - `callback`: Callback to associate with state.
    pub fn with_state<C: 'static>(self, state: S, callback: C) -> Self
    where
        C: StateCallback<S, E> + Send + Sync,
    {
        self.with_state_fn(state, move || callback)
    }

    /// Register a state callback which is `!Send`.
    pub fn with_state_fn<T: 'static, C: 'static>(mut self, state: S, callback: C) -> Self
    where
        C: FnOnce() -> T + Send + Sync,
        T: StateCallback<S, E>,
    {
        self.states.push((
            state,
            SendBoxFnOnce::from(move || Box::new(callback()) as Box<StateCallback<S, E>>),
        ));

        self
    }

    /// Registers a `System` into this application's `GameData`.
    ///
    /// # Parameters
    ///
    /// * `system`: The `System` to register.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with_system<N, Sys>(self, system: Sys, name: N, deps: &[N]) -> Self
    where
        N: Into<String> + Clone,
        Sys: for<'sys_local> System<'sys_local> + Send + 'static,
    {
        let name = name.into();
        let deps = deps
            .iter()
            .map(|dep| dep.clone().into())
            .collect::<Vec<String>>();
        self.with_bundle_fn(move || SystemInjectionBundle::new(system, name, deps))
    }

    /// Registers a `System` to run in a `CustomDispatcherState`.
    ///
    /// This will run the system once in a dedicated `State`, allowing you to inspect the effects of
    /// the system after setting up the world to a desired state.
    ///
    /// # Parameters
    ///
    /// * `system`: The `System` to register.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with_state_system<N, Sys>(self, state: S, system: Sys, name: N, deps: &[N]) -> Self
    where
        N: Into<String> + Clone,
        Sys: for<'sys_local> System<'sys_local> + Send + Sync + 'static,
    {
        let name = name.into();

        let deps = deps
            .iter()
            .map(|dep| dep.clone().into())
            .collect::<Vec<String>>();

        self.with_state_fn(state, move || {
            CustomDispatcherStateBuilder::new()
                .with(
                    system,
                    &name,
                    &deps.iter().map(|dep| dep.as_ref()).collect::<Vec<&str>>(),
                ).build()
        })
    }

    /// Switches to the given state in the sequence.
    ///
    /// # Parameters
    ///
    /// * `state`: State to switch to.
    pub fn do_state(self, state: S) -> Self {
        self.do_fn(move |_| Step::Trans(Trans::Switch(state.clone())))
    }

    /// Registers a function to run on the `World`.
    ///
    /// # Parameters
    ///
    /// * `func`: Function to execute against the world.
    pub fn do_fn<F, T>(mut self, mut func: F) -> Self
    where
        F: 'static + FnMut(&mut World) -> T + Send + Sync,
        T: 'static,
        Step<S>: From<T>,
    {
        self.functions
            .push_back(Box::new(move |world| Step::from(func(world))));
        self
    }

    /// Wait for the given number of updates before progressing.
    pub fn do_wait(self, count: usize) -> Self {
        if count == 0 {
            return self;
        }

        let mut waiter = Waiter(count);
        return self.do_fn(move |_| waiter.next());

        struct Waiter(usize);

        impl Waiter {
            fn next<S>(&mut self) -> Step<S> {
                let c = self.0.saturating_sub(1);

                if c == 0 {
                    return Step::Next;
                }

                self.0 = c;
                Step::None
            }
        }
    }

    /// Marks that this application uses the `RenderBundle`.
    ///
    /// **Note:** There is a `.with_render_bundle()` convenience function if you just need the
    /// `RenderBundle` with predefined parameters.
    ///
    /// This is used to avoid a window initialization race condition that causes tests to fail.
    /// See <https://github.com/tomaka/glutin/issues/1038>.
    pub fn mark_render(mut self) -> Self {
        self.render = true;
        self
    }

    /// Convenience function that returns a `DisplayConfig`.
    ///
    /// The configuration uses the following parameters:
    ///
    /// * `title`: As provided.
    /// * `fullscreen`: `false`
    /// * `dimensions`: `Some((800, 600))`
    /// * `min_dimensions`: `Some((400, 300))`
    /// * `max_dimensions`: `None`
    /// * `vsync`: `true`
    /// * `multisampling`: `0` (disabled)
    /// * `visibility`: As provided.
    ///
    /// This is exposed to allow external crates a convenient way of obtaining display
    /// configuration.
    ///
    /// # Parameters
    ///
    /// * `title`: Window title.
    /// * `visibility`: Whether the window should be visible.
    pub fn display_config(title: String, visibility: bool) -> DisplayConfig {
        DisplayConfig {
            title,
            fullscreen: false,
            dimensions: Some((SCREEN_WIDTH, SCREEN_HEIGHT)),
            min_dimensions: Some((SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2)),
            max_dimensions: None,
            vsync: true,
            multisampling: 0, // Must be multiple of 2, use 0 to disable
            visibility,
        }
    }

    /// Convenience function that returns a `PipelineBuilder`.
    ///
    /// The pipeline is built from the following:
    ///
    /// * Black clear target.
    /// * `DrawFlat2D` pass with transparency.
    /// * `DrawUi` pass.
    ///
    /// This is exposed to allow external crates a convenient way of obtaining a render pipeline.
    pub fn pipeline() -> DefaultPipeline {
        Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target([0., 0., 0., 0.], 0.)
                .with_pass(DrawFlat2D::new().with_transparency(
                    ColorMask::all(),
                    ALPHA,
                    Some(DepthMode::LessEqualWrite),
                )).with_pass(DrawUi::new()),
        )
    }
}

#[cfg(test)]
mod test {
    use amethyst::{
        self,
        assets::{self, Asset, AssetStorage, Handle, Loader, ProcessingState, Processor},
        core::bundle::{self, SystemBundle},
        ecs::prelude::*,
        prelude::*,
        renderer::ScreenDimensions,
        ui::FontAsset,
    };

    use super::AmethystApplication;
    use crate::{EffectReturn, FunctionState, ReturnState};
    #[cfg(feature = "graphics")]
    use MaterialAnimationFixture;
    #[cfg(feature = "graphics")]
    use SpriteRenderAnimationFixture;

    #[test]
    fn bundle_build_is_ok() {
        assert!(
            AmethystApplication::<(), _, _>::blank()
                .with_bundle(BundleZero)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn load_multiple_bundles() {
        assert!(
            AmethystApplication::<(), _, _>::blank()
                .with_bundle(BundleZero)
                .with_bundle(BundleOne)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn assertion_when_resource_is_added_succeeds() {
        let assertion_fn = |world: &mut World| {
            world.read_resource::<ApplicationResource>();
            world.read_resource::<ApplicationResourceNonDefault>();
        };

        assert!(
            AmethystApplication::<(), _, _>::blank()
                .with_bundle(BundleZero)
                .with_bundle(BundleOne)
                .do_fn(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    #[should_panic(expected = "Failed to run Amethyst application")]
    fn assertion_when_resource_is_not_added_should_panic() {
        let assertion_fn = |world: &mut World| {
            // Panics if `ApplicationResource` was not added.
            world.read_resource::<ApplicationResource>();
        };

        assert!(
            AmethystApplication::<(), _, _>::blank()
                // without BundleOne
                .do_fn(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn assertion_switch_with_loading_state_with_add_resource_succeeds() {
        assert!(
            AmethystApplication::blank()
                .with_state("loading", LoadingState::new("assertion"))
                .with_state(
                    "assertion",
                    FunctionState::new(|world: &mut World| {
                        world.read_resource::<LoadResource>();
                    })
                ).do_state("loading")
                .run()
                .is_ok()
        );
    }

    #[test]
    fn assertion_push_with_loading_state_with_add_resource_succeeds() {
        // Alternative to embedding the `FunctionState` is to switch to a `ReturnState` but still
        // provide the assertion function
        let assertion_fn = |world: &mut World| {
            world.read_resource::<LoadResource>();
        };

        assert!(
            AmethystApplication::blank()
                .with_state("loading", LoadingState::new("return"))
                .with_state("return", ReturnState(Trans::Pop))
                .do_state("loading")
                .do_wait(1)
                // then assert
                .do_fn(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    #[should_panic(expected = "Failed to run Amethyst application")]
    fn assertion_switch_with_loading_state_without_add_resource_should_panic() {
        let switch_state = SwitchState::new("assertion");

        let assertion = FunctionState::new(|world: &mut World| {
            world.read_resource::<LoadResource>();
        });

        assert!(
            AmethystApplication::blank()
                .with_state("state", switch_state)
                .with_state("assertion", assertion)
                .do_state("state")
                .do_wait(1)
                .run()
                .is_ok()
        );
    }

    #[test]
    #[should_panic(expected = "Failed to run Amethyst application")]
    fn assertion_push_with_loading_state_without_add_resource_should_panic() {
        // Alternative to embedding the `FunctionState` is to switch to a `PopState` but still
        // provide the assertion function
        let assertion_fn = |world: &mut World| {
            world.read_resource::<LoadResource>();
        };

        assert!(
            AmethystApplication::blank()
                .with_state((), ReturnState(Trans::Pop))
                .do_fn(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn game_data_must_update_before_assertion() {
        let effect_fn = |world: &mut World| {
            let handles = vec![
                AssetZeroLoader::load(world, AssetZero(10)).unwrap(),
                AssetZeroLoader::load(world, AssetZero(20)).unwrap(),
            ];

            world.add_resource::<Vec<AssetZeroHandle>>(handles);
        };
        let assertion_fn = |world: &mut World| {
            let asset_zero_handles = world.read_resource::<Vec<AssetZeroHandle>>();

            let store = world.read_resource::<AssetStorage<AssetZero>>();
            assert_eq!(Some(&AssetZero(10)), store.get(&asset_zero_handles[0]));
            assert_eq!(Some(&AssetZero(20)), store.get(&asset_zero_handles[1]));
        };

        assert!(
            AmethystApplication::<(), _, _>::blank()
                .with_bundle(BundleAsset)
                .do_fn(effect_fn)
                .do_fn(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn execution_order_is_setup_state_effect_assertion() {
        struct Setup;
        let setup_fns = |world: &mut World| world.add_resource(Setup);

        let inner = FunctionState::new(|world: &mut World| {
            // Panics if setup is not run before this.
            world.read_resource::<Setup>();
        });
        let loading = LoadingState::new("inner");

        let effect_fn = |world: &mut World| {
            // If `LoadingState` is not run before this, this will panic
            world.read_resource::<LoadResource>();

            let handles = vec![AssetZeroLoader::load(world, AssetZero(10)).unwrap()];
            world.add_resource(handles);
        };
        let assertion_fn = |world: &mut World| {
            let asset_zero_handles = world.read_resource::<Vec<AssetZeroHandle>>();

            let store = world.read_resource::<AssetStorage<AssetZero>>();
            assert_eq!(Some(&AssetZero(10)), store.get(&asset_zero_handles[0]));
        };

        assert!(
            AmethystApplication::blank()
                .with_bundle(BundleAsset)
                .with_state("loading", loading)
                .with_state("inner", inner)
                .do_state("loading")
                .do_fn(setup_fns)
                .do_fn(effect_fn)
                .do_fn(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn base_application_can_load_ui() {
        let assertion_fn = |world: &mut World| {
            // Next line would panic if `UiBundle` wasn't added.
            world.read_resource::<AssetStorage<FontAsset>>();
            // `.base()` should add `ScreenDimensions` as this is necessary for `UiBundle` to
            // initialize properly.
            world.read_resource::<ScreenDimensions>();
        };

        assert!(
            AmethystApplication::<(), _, _>::ui_base::<String, String>()
                .do_fn(assertion_fn)
                .run()
                .is_ok()
        );
    }

    #[test]
    #[cfg(feature = "graphics")]
    fn render_base_application_can_load_material_animations() {
        assert!(
            AmethystApplication::render_base(
                "render_base_application_can_load_material_animations",
                false
            ).do_fn(MaterialAnimationFixture::effect)
            .do_fn(MaterialAnimationFixture::assertion)
            .run()
            .is_ok()
        );
    }

    #[test]
    #[cfg(feature = "graphics")]
    fn render_base_application_can_load_sprite_render_animations() {
        assert!(
            AmethystApplication::render_base(
                "render_base_application_can_load_sprite_render_animations",
                false
            ).do_fn(SpriteRenderAnimationFixture::effect)
            .do_fn(SpriteRenderAnimationFixture::assertion)
            .run()
            .is_ok()
        );
    }

    #[test]
    fn with_system_runs_system_every_tick() {
        let effect_fn = |world: &mut World| {
            let entity = world.create_entity().with(ComponentZero(0)).build();

            world.add_resource(EffectReturn(entity));
        };

        fn get_component_zero_value(world: &mut World) -> i32 {
            let entity = world.read_resource::<EffectReturn<Entity>>().0.clone();

            let component_zero_storage = world.read_storage::<ComponentZero>();
            let component_zero = component_zero_storage
                .get(entity)
                .expect("Entity should have a `ComponentZero` component.");

            component_zero.0
        };

        assert!(
            AmethystApplication::<(), _, _>::blank()
                .with_system(SystemEffect, "system_effect", &[])
                .do_fn(effect_fn)
                .do_fn(|world| assert_eq!(1, get_component_zero_value(world)))
                .do_fn(|world| assert_eq!(2, get_component_zero_value(world)))
                .run()
                .is_ok()
        );
    }

    #[test]
    fn with_system_invoked_twice_should_not_panic() {
        AmethystApplication::<(), _, _>::blank()
            .with_system(SystemZero, "zero", &[])
            .with_system(SystemOne, "one", &["zero"]);
    }

    #[test]
    fn with_system_single_runs_system_once() {
        let assertion_fn = |world: &mut World| {
            let entity = world.read_resource::<EffectReturn<Entity>>().0.clone();

            let component_zero_storage = world.read_storage::<ComponentZero>();
            let component_zero = component_zero_storage
                .get(entity)
                .expect("Entity should have a `ComponentZero` component.");

            // If the system ran, the value in the `ComponentZero` should be 1.
            assert_eq!(1, component_zero.0);
            Trans::None
        };

        assert!(
            AmethystApplication::<&str, _, _>::blank()
                .with_state_system("effect", SystemEffect, "system_effect", &[])
                .do_state("effect")
                .do_fn(|world| {
                    world.register::<ComponentZero>();

                    let entity = world.create_entity().with(ComponentZero(0)).build();
                    world.add_resource(EffectReturn(entity));
                    Trans::None
                }).do_fn(assertion_fn)
                .do_fn(assertion_fn)
                .run()
                .is_ok()
        );
    }

    // Double usage tests
    // If the second call panics, then the setup functions were not executed in the right order.

    #[test]
    fn with_setup_invoked_twice_should_run_in_specified_order() {
        assert!(
            AmethystApplication::<(), _, _>::blank()
                .do_fn(|world| {
                    world.add_resource(ApplicationResource);
                }).do_fn(|world| {
                    world.read_resource::<ApplicationResource>();
                }).run()
                .is_ok()
        );
    }

    #[test]
    fn with_effect_invoked_twice_should_run_in_the_specified_order() {
        assert!(
            AmethystApplication::<(), _, _>::blank()
                .do_fn(|world| {
                    world.add_resource(ApplicationResource);
                }).do_fn(|world| {
                    world.read_resource::<ApplicationResource>();
                }).run()
                .is_ok()
        );
    }

    #[test]
    fn with_assertion_invoked_twice_should_run_in_the_specified_order() {
        assert!(
            AmethystApplication::<(), _, _>::blank()
                .do_fn(|world| {
                    world.add_resource(ApplicationResource);
                }).do_fn(|world| {
                    world.read_resource::<ApplicationResource>();
                }).run()
                .is_ok()
        );
    }

    #[test]
    fn with_state_invoked_twice_should_run_in_the_specified_order() {
        assert!(
            AmethystApplication::blank()
                .with_state(
                    "first",
                    FunctionState::new(|world| {
                        world.add_resource(ApplicationResource);
                    })
                ).with_state(
                    "second",
                    FunctionState::new(|world| {
                        world.read_resource::<ApplicationResource>();
                    })
                ).do_state("first")
                .do_state("second")
                .do_wait(1)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn setup_can_be_invoked_after_with_state() {
        assert!(
            AmethystApplication::blank()
                .with_state(
                    "first",
                    FunctionState::new(|world| {
                        world.add_resource(ApplicationResource);
                    })
                ).do_state("first")
                .do_wait(1)
                .do_fn(|world| {
                    world.read_resource::<ApplicationResource>();
                }).run()
                .is_ok()
        );
    }

    #[test]
    fn with_state_invoked_after_with_resource_should_work() {
        assert!(
            AmethystApplication::blank()
                .with_resource(ApplicationResource)
                .with_state(
                    "first",
                    FunctionState::new(|world| {
                        world.read_resource::<ApplicationResource>();
                    })
                ).run()
                .is_ok()
        );
    }

    // === Resources === //
    #[derive(Debug, Default)]
    struct ApplicationResource;
    #[derive(Debug)]
    struct ApplicationResourceNonDefault;
    #[derive(Debug)]
    struct LoadResource;

    // === States === //
    struct LoadingState<S> {
        next_state: Option<S>,
    }
    impl<S> LoadingState<S> {
        fn new(next_state: S) -> Self {
            LoadingState {
                next_state: Some(next_state),
            }
        }
    }
    impl<S, E> StateCallback<S, E> for LoadingState<S> {
        fn update(&mut self, world: &mut World) -> Trans<S> {
            if let Some(next) = self.next_state.take() {
                world.add_resource(LoadResource);
                return Trans::Switch(next);
            }

            Trans::None
        }
    }

    struct SwitchState<S> {
        next_state: Option<S>,
    }
    impl<S> SwitchState<S> {
        fn new(next_state: S) -> Self {
            SwitchState {
                next_state: Some(next_state),
            }
        }
    }
    impl<S, E> StateCallback<S, E> for SwitchState<S> {
        fn update(&mut self, _: &mut World) -> Trans<S> {
            Trans::Switch(self.next_state.take().unwrap())
        }
    }

    // === Systems === //
    #[derive(Debug)]
    struct SystemZero;
    impl<'s> System<'s> for SystemZero {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    #[derive(Debug)]
    struct SystemOne;
    type SystemOneData<'s> = Read<'s, ApplicationResource>;
    impl<'s> System<'s> for SystemOne {
        type SystemData = SystemOneData<'s>;
        fn run(&mut self, _: Self::SystemData) {}
    }

    #[derive(Debug)]
    struct SystemNonDefault;
    type SystemNonDefaultData<'s> = ReadExpect<'s, ApplicationResourceNonDefault>;
    impl<'s> System<'s> for SystemNonDefault {
        type SystemData = SystemNonDefaultData<'s>;
        fn run(&mut self, _: Self::SystemData) {}

        fn setup(&mut self, res: &mut Resources) {
            // Must be called when we override `.setup()`
            SystemNonDefaultData::setup(res);

            // Need to manually insert this when the resource is `!Default`
            res.insert(ApplicationResourceNonDefault);
        }
    }

    #[derive(Debug)]
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
        fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
            builder.add(SystemZero, "system_zero", &[]);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct BundleOne;
    impl<'a, 'b> SystemBundle<'a, 'b> for BundleOne {
        fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
            builder.add(SystemOne, "system_one", &["system_zero"]);
            builder.add(SystemNonDefault, "system_non_default", &[]);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct BundleAsset;
    impl<'a, 'b> SystemBundle<'a, 'b> for BundleAsset {
        fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
            builder.add(Processor::<AssetZero>::new(), "asset_zero_processor", &[]);
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
    impl From<AssetZero> for Result<ProcessingState<AssetZero>, assets::Error> {
        fn from(asset_zero: AssetZero) -> Result<ProcessingState<AssetZero>, assets::Error> {
            Ok(ProcessingState::Loaded(asset_zero))
        }
    }
    type AssetZeroHandle = Handle<AssetZero>;

    // === System delegates === //
    struct AssetZeroLoader;
    impl AssetZeroLoader {
        fn load(world: &World, asset_zero: AssetZero) -> Result<AssetZeroHandle, amethyst::Error> {
            let loader = world.read_resource::<Loader>();
            Ok(loader.load_from_data(
                asset_zero,
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

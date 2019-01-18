# Testing

Without a doubt, Amethyst contains many concepts for you to understand and remember. During development, normally each concept's types are written in its own module.

To test that these types work properly often requires them to be run in an Amethyst application. By now you *know* that there is much boilerplate required to setting up an application simply to test a single system.

The `amethyst_test` crate provides support to write tests ergonomically and expressively.

The following shows a simple example of testing a `State`. More examples are in following pages.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# extern crate amethyst_test;
#
# use std::marker::PhantomData;
#
# use amethyst_test::prelude::*;
# use amethyst::{
#     ecs::prelude::*,
#     prelude::*,
# };
#
# #[derive(Debug)]
# struct LoadResource;
#
# #[derive(Debug)]
# struct LoadingState;
#
# impl LoadingState {
#     fn new() -> Self {
#         LoadingState
#     }
# }
#
# impl<'a, 'b, E> State<GameData<'a, 'b>, E> for LoadingState
# where
#     E: Send + Sync + 'static,
# {
#     fn update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> Trans<GameData<'a, 'b>, E> {
#         data.data.update(&data.world);
#
#         data.world.add_resource(LoadResource);
#
#         Trans::Pop
#     }
# }
#
#[test]
fn loading_state_adds_load_resource() {
    assert!(
        AmethystApplication::blank()
            .with_state(|| LoadingState::new())
            .with_assertion(|world| {
                world.read_resource::<LoadResource>();
            })
            .run()
            .is_ok()
    );
}
```

## Anatomy of an Amethyst Test Function

The Amethyst application is initialized with one of the following functions, each providing a different set of bundles:

```rust,edition2018,no_run,noplaypen
extern crate amethyst_test;

use amethyst_test::prelude::*;

#[test]
fn test_name() {
    // Start with no bundles
    AmethystApplication::blank();

    // Start with the following bundles:
    //
    // * `TransformBundle`
    // * `InputBundle`
    // * `UiBundle`
    //
    // The type parameters here are the Axis and Action types for the
    // `InputBundle` and `UiBundle`.
    AmethystApplication::ui_base::<String, String>();

    // Start with the following bundles:
    //
    // * `AnimationBundle::<u32, Material>`
    // * `AnimationBundle::<u32, SpriteRender>`
    // * `TransformBundle`
    // * `RenderBundle`
    //
    // If you want the Input and UI bundles, you can use the
    // `.with_ui_bundles::<AX, AC>()` method.
    let visibility = false; // Whether the window should be shown
    AmethystApplication::render_base("test_name", visibility);
}
```

Next, attach the logic for your test using the various `.with_*(..)` methods:

```rust,ignore
#[test]
fn test_name() {
    let visibility = false; // Whether the window should be shown
    AmethystApplication::render_base::<String, String, _>("test_name", visibility)
        .with_bundle(MyBundle::new())                // Registers a bundle.
        .with_bundle_fn(|| MyNonSendBundle::new())   // Registers a `!Send` bundle.
        .with_resource(MyResource::new())            // Adds a resource to the world.
        .with_system(MySystem::new(), "my_sys", &[]) // Registers a system
                                                     // with the main dispatcher

        // These are run in the order they are invoked.
        // You may invoke them multiple times.
        .with_setup(|world| { /* do something */ })
        .with_state(|| MyState::new())
        .with_effect(|world| { /* do something */ })
        .with_assertion(|world| { /* do something */ })
         // ...
}
```

Finally, call `.run()` to run the application. This returns `amethyst::Result<()>`, so you can
wrap it in an `assert!(..);`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst_test;
#
# use amethyst_test::prelude::*;
#
#[test]
fn test_name() {
    let visibility = false; // Whether the window should be shown
    assert!(
        AmethystApplication::render_base("test_name", visibility)
            // ...
            .run()
            .is_ok()
    );
}
```

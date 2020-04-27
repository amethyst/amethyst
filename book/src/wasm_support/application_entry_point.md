# Application Entry Point

These steps update an existing Amethyst application that compiles natively, to support WASM compilation.

<details open>

<summary>1. Update the project to match <code>"wasm"</code> branch dependency updates.</summary>

<div style="padding-left: 40px;">

This step ensures the project runs while using the `"wasm"` branch.

```patch
// === Imports === //
// Replace
-use winit::{MouseButton, VirtualKeyCode};
// With
+use winit::event::{MouseButton, VirtualKeyCode};

// In `main.rs`, add the following imports
+use amethyst::renderer::rendy::hal::command::ClearColor;
+use amethyst::window::{DisplayConfig, EventLoop};

// === main() === //
// Add this before instantiating `game_data`:
+ let event_loop = EventLoop::new();
+ let display_config = DisplayConfig::load(display_config_path)?;
let game_data = GameDataBuilder::default()

// Update how `RenderingBundle` is instantiated
-RenderingBundle::<DefaultBackend>::new()
+RenderingBundle::<DefaultBackend>::new(display_config, &event_loop)

// Update how `RenderToWindow` is instantiated.
-    .with_plugin(
-        RenderToWindow::from_config_path(display_config_path)?
-            .with_clear([0.1, 0.1, 0.1, 1.0]),
-    )
+    .with_plugin(RenderToWindow::new().with_clear(ClearColor {
+        float32: [0.1, 0.1, 0.1, 1.0],
+    }))

// Switch the `run()` method call to one that uses the `winit` event loop.
-game.run();
+game.run_winit_loop(event_loop);
```

The changes are based on:

* Upgrading `winit` from `0.19` to `0.22`
* Upgrading `rendy` from `0.4` to `0.5`

Make sure the project successfully runs:

```bash
cargo run --features "vulkan" # or the appropriate backend
```

</div>

</details>

<details open>

<summary>2. Split <code>run_application()</code> from the <code>main()</code> function.</summary>

<div style="padding-left: 40px;">

This step splits native application initialization from the event loop execution. This is in preparation for the WASM entry point.

The changes in this step include:

* Initializing the rendering bundle and input bindings in `main()`, in a closure.
* Passing that closure to a `run_application` function, which executes the existing logic.

```rust,ignore
#[cfg(not(feature = "wasm"))]
fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let setup_fn = |app_root: &Path, event_loop: &EventLoop<()>| {
        let key_bindings_path = {
            // If you are using the `"sdl_controller"` feature, you can use this
            // conditional. Otherwise just use what's in the `else` block.
            if cfg!(feature = "sdl_controller") {
                app_root.join("config/input_controller.ron")
            } else {
                app_root.join("config/input.ron")
            }
        };
        let bindings =
            <Bindings<StringBindings> as Config>::load(key_bindings_path)?;

        let display_config =
            DisplayConfig::load(app_root.join("config/display.ron"))?;
        let rendering_bundle =
            RenderingBundle::<DefaultBackend>::new(display_config, event_loop);

        Ok((bindings, rendering_bundle))
    };

    run_application(setup_fn)
}

fn run_application<FnSetup>(setup_fn: FnSetup) -> amethyst::Result<()>
where
    FnSetup:
        FnOnce(
            &Path,
            &EventLoop<()>,
        )
            -> amethyst::Result<(
                Bindings<StringBindings>,
                RenderingBundle<DefaultBackend>
            )>,
{
    let app_root = application_root_dir()?;
    let event_loop = EventLoop::new();

    let (bindings, rendering_bundle) = setup_fn(&app_root, &event_loop)?;

    // existing logic
    // ..

    let game_data = GameDataBuilder::default()
        // existing bundles
        // ..
        .with_bundle(
            // Note: Render plugins are still registered here.
            rendering_bundle
                .with_plugin(RenderToWindow::new().with_clear(ClearColor {
                    float32: [0.1, 0.1, 0.1, 1.0],
                }))
                // other plugins
                // ..
        )?;

    game.run_winit_loop(event_loop);
}
```

Verify that the project successfully runs:

```bash
cargo run --features "vulkan" # or the appropriate backend
```

</div>

</details>

<details open>

<summary>3. Add the WASM application entry point.</summary>

<div style="padding-left: 40px;">

This step adds the entry point that will be called from the web page javascript.

The entry point will use the same `run_application(..)` function from the previous step.

It is beneficial to read through the whole snippet, and make any modifications necessary for your project.

```rust,ignore
#[allow(unused)]
#[cfg(feature = "wasm")]
fn main() {}

#[cfg(feature = "wasm")]
mod wasm {
    use std::path::Path;

    use amethyst::{
        config::Config,
        input::{Axis, Bindings, Button, StringBindings},
        renderer::{types::DefaultBackend, RenderingBundle},
        window::{DisplayConfig, EventLoop},
        winit::event::VirtualKeyCode,
    };
    use wasm_bindgen::prelude::*;
    use web_sys::HtmlCanvasElement;

    /// `MyApplication` builder.
    #[wasm_bindgen]
    #[derive(Debug, Default)]
    pub struct MyAppBuilder {
        /// User supplied canvas, if any.
        canvas_element: Option<HtmlCanvasElement>,
        /// Input bindings data.
        input_bindings_str: Option<String>,
    }

    #[wasm_bindgen]
    impl MyAppBuilder {
        /// Returns a new `MyAppBuilder`.
        pub fn new() -> Self {
            Self::default()
        }

        /// Sets the canvas element for the `MyAppBuilder`.
        pub fn with_canvas(mut self, canvas: HtmlCanvasElement) -> Self {
            self.canvas_element = Some(canvas);
            self
        }

        /// Sets the canvas element for the `MyAppBuilder`.
        pub fn with_input_bindings(mut self, input_bindings_str: String)
        -> Self
        {
            self.input_bindings_str = Some(input_bindings_str);
            self
        }

        pub fn run(self) {
            // Make panic return a stack trace
            crate::init_panic_hook();

            amethyst::start_logger(Default::default());

            log::debug!("canvas element: {:?}", self.canvas_element);

            let dimensions = self
                .canvas_element
                .as_ref()
                .map(|canvas_element| {
                    (canvas_element.width(), canvas_element.height())
                });
            log::debug!("dimensions: {:?}", dimensions);

            let display_config = DisplayConfig {
                dimensions,
                ..Default::default()
            };

            let bindings = if let Some(input_bindings_str) =
                self.input_bindings_str.as_ref()
            {
                <Bindings<StringBindings> as Config>::load_bytes(
                    input_bindings_str.as_bytes()
                ).expect("Failed to deserialize input bindings.")
            } else {
                // Hard coded bindings
                // Update these to be something suitable for your application.

                log::debug!("Using built in bindings.");

                let mut bindings = Bindings::<StringBindings>::new();
                let left_paddle_axis = Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::W),
                    neg: Button::Key(VirtualKeyCode::S),
                };
                let _ = bindings.insert_axis("left_paddle", left_paddle_axis);
                let right_paddle_axis = Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::Up),
                    neg: Button::Key(VirtualKeyCode::Down),
                };
                let _ = bindings.insert_axis("right_paddle", right_paddle_axis);

                bindings
            };

            // The `application_root_dir` parameter is not used as WASM
            // applications don't access the file system.
            let setup_fn = move |_: &Path, event_loop: &EventLoop<()>| {
                let rendering_bundle = RenderingBundle::<DefaultBackend>::new(
                    display_config,
                    event_loop,
                    self.canvas_element,
                );

                Ok((bindings, rendering_bundle))
            };

            let res = super::run_application(setup_fn);
            match res {
                Ok(_) => log::info!("Exited without error"),
                Err(e) => log::error!("Main returned an error: {:?}", e),
            }
        }
    }
}
```

</div>

</details>

There are additional non-Rust files to support compiling the project to a WASM application. These are covered on the next page.

# Introduction 

The most usual way to have your players interact with your game is through your user interface or UI.
In general UI includes all kinds of widgets from images, buttons, progress bars, text, sliders, popup menus, etc.

The API in Amethyst was designed more to provide users with building blocks for the UI and without 
a specific layout system. The reason is that you don't often see, if any, layout systems 
used in games, although are very popular GUI frameworks. 

Please note that not all forementinoted widgets exist in Amethyst yet.

## Setting up the UI

The first thing you need to add to your systems is the [UiBundle](https://docs.amethyst.rs/master/amethyst_ui/struct.UiBundle.html). The `UiBundle` registers
all the needed components,systems and resources in order to be able use the UI. Another **very important thing**
is that you want to add the [InputBundle](https://docs.amethyst.rs/master/amethyst_input/struct.InputBundle.html) 
**before** the `UiBundle`,
otherwise the application will panic, since it is a dependency for the `UiBundle`!

Now you are able to create your widgets! Unfortunately you won't be able to see them. That's why you also need 
to add a plugin to your rendering bundle called [RenderUi](https://docs.amethyst.rs/master/amethyst_ui/struct.RenderUi.html) in order
to draw these widgets.

A minimalistic game data would now look like this:  

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::{
#     GameDataBuilder,
#     input::{InputBundle, StringBindings},
#     renderer::{types::DefaultBackend, RenderingBundle, RenderToWindow},
#     Result,
#     ui::{RenderUi, UiBundle}
# };
# 
# pub fn main() -> Result<()> {
    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config(Default::default())
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderUi::default()),
        )?;
#   Ok(())
# }
```

Make sure that the `InputBundle` and `UiBundle` have same binding types. In this case these
are `StringBindings`.
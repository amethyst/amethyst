# Build your own!

In this chapter we will guide you through building your own button in Amethyst!

### Bulding blocks

The components you can use in order to build your button are as goes:

- [UiTransform](https://docs.amethyst.rs/master/amethyst_ui/struct.UiTransform.html) -
used for positioning your button on the screen (same as Transform but for the UI elements)

- [UiText](https://docs.amethyst.rs/master/amethyst_ui/struct.UiText.html) -
if you want your button to have any text displayed

- [UiImage](https://docs.amethyst.rs/master/amethyst_ui/enum.UiImage.html) -
if you want your button to display a texture


You don't have to use all three at the same time of course but variations of two (`UiTransform` is always needed!).

### Creating the `UiTransform`

One way of defining a `UiTransform` is like so:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ui::{Anchor, UiTransform};

let ui_transform = UiTransform::new(
    String::from("simple_button"), // id
    Anchor::Middle,                // anchor
    Anchor::Middle,                // pivot
    0f32,                          // x
    0f32,                          // y
    0f32,                          // z
    100f32,                        // width
    30f32,                         // height
);
```

The `id` field of the transform is basically like the name. You can use this in combination with the
[UiFinder](https://docs.amethyst.rs/master/amethyst_ui/struct.UiFinder.html) to fetch the transfrom through a system.

Assuming the entity has no parent, whatever is set as the `anchor` field will be placed relative to the screen. In our case
we set it to `Anchor::Middle` and it will be drawn in the middle of the screen. The `pivot` field will center the widget
relative to itself - this in turn is the reason why our `x` and `y` fields are `0f32`. The `z` field of this struct 
is used for "depth" ordering of the ui elements.

The `width` and `height` fields are also important. They represent the area that will register the events like hovering over 
with the mouse, clicking and dragging. If you built the entity with the `UiText` component this also determines if the text will be rendered, 
meaning you need
to set the area big enough for the text to fit in!


### Creating the `UiText`

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::assets::{AssetStorage, Loader};
# use amethyst::ui::{Anchor, FontAsset, get_default_font, LineMode, UiText};
# use amethyst::prelude::{World, WorldExt};
#
# fn some_function(world: &mut World) {
#    let font_handle = {
#        let loader = world.read_resource::<Loader>();
#        let font_storage = world.read_resource::<AssetStorage<FontAsset>>();
#        get_default_font(&loader, &font_storage)
#    };
    let ui_text = UiText::new(
        font_handle,                   // font
        String::from("Simple Button"), // text
        [1.0, 1.0, 1.0, 0.5],          // color
        25f32,                         // font_size
        LineMode::Single,              // line mode
        Anchor::Middle,                // alignment
    );
# }
```
The `text` field of this struct is pretty self explanatory. It's what you would want to access if
you were to dynamically change the text on the screen through systems.

You also need to load a specific font handle and provide it for the text.

If you had some state implemented you can create the button on its `on_start` method:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::assets::{AssetStorage, Loader};
# use amethyst::ui::{
#     Anchor, FontAsset, get_default_font, LineMode, UiText, UiTransform,
# };
# use amethyst::prelude::{Builder, GameData, SimpleState, SimpleTrans, StateData, Trans, World, WorldExt};
#
# pub struct State;
#
# impl SimpleState for State {
#
fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
    let world = data.world;

    /* Create the transform */
    let ui_transform = UiTransform::new(
        // ...
#         String::from("simple_button"), // id
#         Anchor::Middle,                // anchor
#         Anchor::Middle,                // pivot
#         0f32,                          // x
#         0f32,                          // y
#         0f32,                          // z
#         100f32,                        // width
#         30f32,                         // height
    );
#
#    let font_handle = {
#        let loader = world.read_resource::<Loader>();
#        let font_storage = world.read_resource::<AssetStorage<FontAsset>>();
#        get_default_font(&loader, &font_storage)
#    };

    /* Create the text */
    let ui_text = UiText::new(
        // ...
#       font_handle,                   // font
#       String::from("Simple Button"), // text
#       [1.0, 1.0, 1.0, 0.5],          // color
#       25f32,                         // font_size
#       LineMode::Single,
#       Anchor::Middle,
    );

    /* Building the entity */
    let _ = world.create_entity()
        .with(ui_transform)
        .with(ui_text)
        .build();
}
#
#     fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
#         Trans::None
#     }
# }
```

It is recommended to keep the entity either in your state or some kind of resource so you
can hide or delete it when you change the states (like changing menus)!

If you were to run this you would get a button in the middle of the screen saying `"Simple Button"`, but
you won't be able to interact with it (which doesn't actually make it a button yet)!

### Interacting with the button!

In order for the ui to generate events you need to add an [Interactable](https://docs.amethyst.rs/master/amethyst_ui/struct.Interactable.html) 
component to your entity (either when building it or dynamically).

This will not work if the entity doesn't
have a `UiTransform` component!

The code snippet would look like this now:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::assets::{AssetStorage, Loader};
# use amethyst::ui::{
#     Anchor, FontAsset, get_default_font, LineMode, UiText, UiTransform, Interactable,
# };
# use amethyst::ecs::{Builder, World, WorldExt};
# use amethyst::prelude::{GameData, SimpleTrans, StateData};
#
# fn some_function(world: &mut World) {
#    let ui_transform = UiTransform::new(
#         String::from("simple_button"), // id
#         Anchor::Middle,                // anchor
#         Anchor::Middle,                // pivot
#         0f32,                          // x
#         0f32,                          // y
#         0f32,                          // z
#         100f32,                        // width
#         30f32,                         // height
#    );
#
#    let font_handle = {
#        let loader = world.read_resource::<Loader>();
#        let font_storage = world.read_resource::<AssetStorage<FontAsset>>();
#        get_default_font(&loader, &font_storage)
#    };
#    /* Create the text */
#    let ui_text = UiText::new(
#       font_handle,                   // font
#       String::from("Simple Button"), // text
#       [1.0, 1.0, 1.0, 0.5],          // color
#       25f32,                         // font_size
#       LineMode::Single,
#       Anchor::Middle,
#    );

let _ = world.create_entity()
    .with(ui_transform)
    .with(ui_text)
    .with(Interactable)
    .build();
# }
```

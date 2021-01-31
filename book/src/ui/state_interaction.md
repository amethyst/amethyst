## State Interaction

Let's declare our state, and call it `MenuState`:

```rust
# use amethyst::ecs::Entity;
# 
#[derive(Default)]
pub struct MenuState {
    button: Option<Entity>,
}
```

We give it a field named `button` which will hold an entity wrapped in
an `Option<T>`. This simplifies things since we can now derive `Default`
trait on it and we can make it as our initial state that the application
will start off as.

It will also serve to hold our ui entity.

In our `on_start` method of this state we can create the button as shown in
previous chapters, but here we will save the entity in our struct:

```rust
# use amethyst::{
#  assets::{AssetStorage,  DefaultLoader, Loader},
# 	ecs::{Entity, World},
# 	ui::{Anchor, FontAsset, Interactable, LineMode, TtfFormat, UiText, UiTransform},
# 	prelude::*,
# };
#
# #[derive(Default)]
# pub struct MenuState {
#   button: Option<Entity>,
# }
impl SimpleState for MenuState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;

        /* Create the transform */
        let ui_transform = UiTransform::new(
        // ...
# 	    String::from("simple_button"), // id
#       Anchor::Middle,                // anchor
#       Anchor::Middle,                // pivot
#       0f32,                          // x
#       0f32,                          // y
#       0f32,                          // z
#       100f32,                        // width
#       30f32,                         // height
        );

        /* Create the text */
        #       let font_handle = resources.get::<DefaultLoader>().load(
        #       "font/square.ttf",
        #       TtfFormat,
        #       (),
        #       resources.get().unwrap(),
        #       );
        #
        let ui_text = UiText::new(
        // ...
#       font_handle,                      // font
#       String::from("Simple Button"),    // text
#       [1.0f32, 1.0f32, 1.0f32, 0.5f32], // color
#       25f32,                            // font_size
#       LineMode::Single,                 // line_mode
#       Anchor::Middle,                   // align
        );

        /* Building the entity */
        let btn = world
            .push((ui_transform,ui_text,Interactable));

        /* Saving the button in our state struct */
        self.button = Some(btn);
    }
}
```

All the input received will be handled in the [handle\_event](https://docs.amethyst.rs/master/amethyst/trait.State.html#method.handle_event)
method of our state:

```rust
# use amethyst::{
#   assets::{AssetStorage,  DefaultLoader, Loader},
#   ecs::{Entity, World},
#   ui::{Anchor, FontAsset, Interactable, LineMode, TtfFormat, UiEventType, UiText, UiTransform},
#   prelude::*,
#   StateEvent,
# };
#
# #[derive(Default)]
# pub struct MenuState {
#   button: Option<Entity>,
# }
impl SimpleState for MenuState {
    #   fn on_start(&mut self, data: StateData<'_, GameData>) {
    // ...
    #       let world = data.world;
    #
    #
    #       /* Create the transform */
    #       let ui_transform = UiTransform::new(
    #           String::from("simple_button"), // id
    #           Anchor::Middle,                // anchor
    #           Anchor::Middle,                // pivot
    #           0f32,                          // x
    #           0f32,                          // y
    #           0f32,                          // z
    #           100f32,                        // width
    #           30f32,                         // height
    #       );
    #
    #       /* Create the text */
    #       let font_handle = resources.get::<DefaultLoader>().load(
    #          "font/square.ttf",
    #          TtfFormat,
    #          (),
    #          resources.get().unwrap(),
    #       );
    #
    #       let ui_text = UiText::new(
    #           font_handle,                      // font
    #           String::from("Simple Button"),    // text
    #           [1.0f32, 1.0f32, 1.0f32, 0.5f32], // color
    #           25f32,                            // font_size
    #           LineMode::Single,                 // line_mode
    #           Anchor::Middle,                   // align
    #       );
    #
    #       /* Building the entity */
    #       let btn = world
    #        .push((ui_transform,ui_text,Interactable));
    #
    #       /* Saving the button in our state struct */
    #       self.button = Some(btn);
    #   }

    fn handle_event(&mut self, _data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Ui(ui_event) = event {
            let is_target = ui_event.target == self.button.unwrap();

            match ui_event.event_type {
                UiEventType::Click if is_target => { /* . . . */ }
                _ => {
                    return SimpleTrans::None;
                }
            };
        }

        SimpleTrans::None
    }
}
```

We only care about the `UiEvent`s here, that's why we can use the `if-let` pattern.
Then we check if the ui target is the same as our saved entity, in this case it
surely is since we've only built one entity. After there's a check for click
event and an additional if statement for our button entity. If it goes well it will
enter that branch.

In this branch you can do whatever you like, either quit if you have a `QUIT` button
and the user clicks on it, in that case we would return a `Trans::Quit`, otherwise
probably something else.

Let's assume something was pushed on top our `MenuState` we would need these two methods:

- [on\_pause](https://docs.amethyst.rs/master/amethyst/trait.State.html#method.on_pause)

- [on\_resume](https://docs.amethyst.rs/master/amethyst/trait.State.html#method.on_resume)

Upon pushing another state the `on_pause` method will run - here we can hide our button.
The way we do that is by adding a [Hidden](https://docs.amethyst.rs/master/amethyst_core/struct.Hidden.html)
component to our button:

```rust
# use amethyst::{
#   assets::{AssetStorage,  DefaultLoader, Loader},
#   core::Hidden,
#   ecs::{Entity, World},
#   ui::{Anchor, FontAsset, Interactable, LineMode, TtfFormat, UiEventType, UiText, UiTransform},
#   prelude::*,
#   StateEvent
# };
#
# #[derive(Default)]
# pub struct MenuState {
#    button: Option<Entity>,
# }
impl SimpleState for MenuState {
    // ...
    #   fn on_start(&mut self, data: StateData<'_, GameData>) {
    #       let world = data.world;
    #
    #
    #       /* Create the transform */
    #       let ui_transform = UiTransform::new(
    #           String::from("simple_button"), // id
    #           Anchor::Middle,                // anchor
    #           Anchor::Middle,                // pivot
    #           0f32,                          // x
    #           0f32,                          // y
    #           0f32,                          // z
    #           100f32,                        // width
    #           30f32,                         // height
    #       );
    #
    #       /* Create the text */
    #       let font_handle = resources.get::<DefaultLoader>().load(
    #           "font/square.ttf",
    #           TtfFormat,
    #           (),
    #           resources.get().unwrap(),
    #       );
    #
    #       let ui_text = UiText::new(
    #           font_handle,                      // font
    #           String::from("Simple Button"),    // text
    #           [1.0f32, 1.0f32, 1.0f32, 0.5f32], // color
    #           25f32,                            // font_size
    #           LineMode::Single,                 // line_mode
    #           Anchor::Middle,                   // align
    #       );
    #
    #       /* Building the entity */
    #       let btn = world
    #        .push((ui_transform,ui_text,Interactable));
    #
    #       /* Saving the button in our state struct */
    #       self.button = Some(btn);
    #   }
    #
    #   fn handle_event(
    #       &mut self,
    #       _data: StateData<'_, GameData>,
    #       event: StateEvent) -> SimpleTrans {
    #       if let StateEvent::Ui(ui_event) = event {
    #           let is_target = ui_event.target == self.button.unwrap();
    #
    #            match ui_event.event_type {
    #               UiEventType::Click if is_target => {
    #               /* . . . */
    #               },
    #               _ => {
    #                   return SimpleTrans::None;
    #               },
    #           };
    #       }
    #
    #       SimpleTrans::None
    #   }

    fn on_pause(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;
        let mut hiddens = world.write_storage::<Hidden>();

        if let Some(btn) = self.button {
            let _ = hiddens.insert(btn, Hidden);
        }
    }
}
```

The same goes for `on_resume` if we actually want to redisplay the button:

```rust
# use amethyst::{
#   assets::{AssetStorage,  DefaultLoader, Loader},
#   core::Hidden,
#   ecs::{Entity, World},
#   ui::{Anchor, FontAsset, Interactable, LineMode, TtfFormat, UiEventType, UiText, UiTransform},
#   prelude::*,
#   StateEvent
# };
#
# #[derive(Default)]
# pub struct MenuState {
#   button: Option<Entity>,
# }
impl SimpleState for MenuState {
    // ...
    #   fn on_start(&mut self, data: StateData<'_, GameData>) {
    #       let world = data.world;
    #
    #
    #       /* Create the transform */
    #       let ui_transform = UiTransform::new(
    #           String::from("simple_button"), // id
    #           Anchor::Middle,                // anchor
    #           Anchor::Middle,                // pivot
    #           0f32,                          // x
    #           0f32,                          // y
    #           0f32,                          // z
    #           100f32,                        // width
    #           30f32,                         // height
    #       );
    #
    #       /* Create the text */
    #       let font_handle = resources.get::<DefaultLoader>().load(
    #           "font/square.ttf",
    #       );
    #
    #       let ui_text = UiText::new(
    #           font_handle,                      // font
    #           String::from("Simple Button"),    // text
    #           [1.0f32, 1.0f32, 1.0f32, 0.5f32], // color
    #           25f32,                            // font_size
    #           LineMode::Single,                 // line_mode
    #           Anchor::Middle,                   // align
    #       );
    #
    #       /* Building the entity */
    #       let btn = world
    #        .push((ui_transform,ui_text,Interactable));
    #
    #       /* Saving the button in our state struct */
    #           self.button = Some(btn);
    #   }
    #
    #   fn handle_event(
    #   &mut self,
    #   _data: StateData<'_, GameData>,
    #   event: StateEvent) -> SimpleTrans {
    #       if let StateEvent::Ui(ui_event) = event {
    #           let is_target = ui_event.target == self.button.unwrap();
    #
    #           match ui_event.event_type {
    #               UiEventType::Click if is_target => {
    #                   /* . . . */
    #               },
    #               _ => {
    #                   return SimpleTrans::None;
    #               },
    #           };
    #       }
    #
    #       SimpleTrans::None
    #   }
    #
    #   fn on_pause(&mut self, data: StateData<'_, GameData>) {
    #   let world = data.world;
    #   let mut hiddens = world.write_storage::<Hidden>();
    #
    #   if let Some(btn) = self.button {
    #      let _ = hiddens.insert(btn, Hidden);
    #   }
    # }

    fn on_resume(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;
        let mut hiddens = world.write_storage::<Hidden>();

        if let Some(btn) = self.button {
            let _ = hiddens.remove(btn);
        }
    }
}
```

This should provide you with the basic knowledge on building the UI.

## State Interaction

Let's declare our state, and call it `MenuState`: 

```rust,edition2018,no_run,noplaypen
# use amethyst::ecs::Entity;

#[derive(Default)]
pub struct MenuState {
    button: Option<Entity>,
}
```

We give it a field named `button` which will hold an entity wrapped in
an `Option<T>`. This simplifies things since we can now derive `Default`
trait on it and we can make it as our initial state that the application
will start off as.

It will also serve to hold our ui entitiy.

In our `on_start` method of this state we can create the button as shown in
previous chapters, but here we will save the entitiy in our struct: 

```rust,edition2018,no_run,noplaypen
# use amethyst::ui::{UiTransform, UiText, Interactable};

fn on_start(&mut self, data: StateData<()>) {
    let world = data.world; 

    /* Create the transform */ 
    let ui_transform = UiTransform { . . . };
    
    /* Create the text */
    let ui_text = UiText { . . . };

    /* Building the entity */
   	let btn = world.create_entity()
        .with(ui_transform)
        .with(ui_text)
	.with(Interactable)
        .build();

	/* Saving the button in our state struct */
	self.button = Some(btn);
}
```

All the input received will be handled in the [handle_event](https://docs.amethyst.rs/master/amethyst/trait.State.html#method.handle_event)
method of our state: 

```rust,edition2018,no_run,noplaypen
# use amethyst::{StateData, GameData, SimpleTrans};

fn handle_event(
	&mut self, 
	_data: StateData<'_, GameData<'_, '_>>, 
	event: StateEvent) -> SimpleTrans {
	if let StateEvent::Ui(ui_event) = event {

		let is_target = ui_event.target == self.button.unwrap();

		match ui_event.event_type {
			UiEventType::Click if is_target => { 
				/* . . . */
			},
			_ => {
				return SimpleTrans::None; 
			},  
		};
	}

	SimpleTrans::None
}
```
We only care about the `UiEvent`s here, that's why we can use the `if-let` pattern.
Then we check if the ui target is the same as our saved entity, in this case it
surely is since we've only built one entity. After there's a check for click
event and an additional if statment for our button entity. If it goes well it will
enter that branch.  

In this branch you can do whatever you like, either quit if you have a `QUIT` button
and the user clicks on it, in that case we would return a `Trans::Quit`, otherwise
probably something else.

Let's assume something was pushed on top our `MenuState` we would need these two methods: 

- [on_pause](https://docs.amethyst.rs/master/amethyst/trait.State.html#method.on_pause)

- [on_resume](https://docs.amethyst.rs/master/amethyst/trait.State.html#method.on_resume)


Upon pushing another state the `on_pause` method will run - here we can hide our button.
The way we do that is by adding a [Hidden](https://docs.amethyst.rs/master/amethyst_core/struct.Hidden.html)
component to our button:

```rust,edition2018,no_run,noplaypen
# use amethyst::core::Hidden;

fn on_pause(&mut self, data: StateData<'_, GameData<'_, '_>>) {
	let world = data.world;	
	let mut hiddens = world.write_storage::<Hidden>();
	
	if let Some(btn) = self.button {
		let _ = hiddens.insert(btn, Hidden);
	}
}
```

The same goes for `on_resume` if we actually want to redisplay the button:

```rust,edition2018,no_run,noplaypen
# use amethyst::{StateData, GameData};
# use amethyst::core::Hidden;

fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
    let world = data.world; 	
    let mut hiddens = world.write_storage::<Hidden>();
	
    if let Some(btn) = self.button {
        let _ = hiddens.remove(btn);
    }
}
```

This should provide you with the basic knowledge on building the UI.

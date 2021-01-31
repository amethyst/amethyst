# System

## What is a `System`?

A system is where the logic of the game is executed. In practice, it consists of a struct implementing a function executed on every iteration of the game loop, and taking as an argument data about the game.

Systems can be seen as a small unit of logic. All systems are run by the engine together (even in parallel when possible), and do a specialized operation on one or a group of entities.

## Structure

A system struct is a structure implementing the trait `amethyst::ecs::System`.

Here is a simple example implementation:

```rust
use amethyst::ecs::{ParallelRunnable, System};

struct MyFirstSystem;

impl System for MyFirstSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(SystemBuilder::new("MyFirstSystem").build(|_, _, _, _| println!("Hello!")))
    }
}
```

This system will, on every iteration of the game loop, print "Hello!" in the console. This is a pretty boring system as it does not interact at all with the game. Let us spice it up a bit.

## Accessing the context of the game

Using `SystemBuilder` requires you to specify the resource and component access requirements of the system using `read_resource`, `write_resource`, `read_component` and `write_component` (you can also use `with_query` but we'll get to that later.)  Refer to the [Legion SystemBuilder docs][sb] for more information.  A system may also have local data stored in its own struct.

```rust
use amethyst::core::timing::Time;
use amethyst::ecs::{ParallelRunnable, System};

struct TimeSystem;

impl System for TimeSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TimeSystem")
                .read_resource::<Time>()
                .build(|_, _, time, _| {
                    println!("{}", data.delta_seconds());
                }),
        )
    }
}
```

Here, we get the `amethyst::core::timing::Time` resource to print in the console the time elapsed between two frames. Nice! But that's still a bit boring.

## Manipulating storages

Once you have access to a storage, you can use them in different ways.

### Getting a component of a specific entity

Sometimes, it can be useful to get a component in the storage for a specific entity. This can easily be done using the `get` or, for mutable storages, `get_mut` methods.

```rust
use amethyst::core::Transform;
use amethyst::ecs::{Entity, System};

struct WalkPlayerUp {
    player: Entity,
}

impl System for WalkPlayerUp {
.write_component::<Transform>()

    fn run(&mut self, mut transforms: Self::SystemData) {
        transforms
            .get_mut(self.player)
            .unwrap()
            .prepend_translation_y(0.1);
    }
}
```

This system makes the player go up by 0.1 unit every iteration of the game loop! To identify what entity the player is, we stored it beforehand in the system's struct. Then, we get its `Transform` from the transform storage, and move it along the Y axis by 0.1.

> A transform is a very common structure in game development. It represents the position, rotation and scale of an object in the game world. You will use them a lot, as they are what you need to change when you want to move something around in your game.

However, this approach is pretty rare because most of the time you don't know what entity you want to manipulate, and in fact you may want to apply your changes to multiple entities.

### Getting all entities with specific components

Most of the time, you will want to perform logic on all entities with a specific component, or even all entities with a selection of components.

This is possible using the `join` method. You may be familiar with joining operations if you have ever worked with databases. The `join` method takes multiple storages, and iterates over all entities that have a component in each of those storages.
It works like an "AND" gate. It will return an iterator containing a tuple of all the requested components if they are **ALL** on the same entity.

If you join with components A, B and C, only the entities that have **ALL** those components will be considered.

Needless to say that you can use it with only one storage to iterate over all entities with a specific component.

Keep in mind that **the `join` method is only available by importing `amethyst::ecs::Join`**.

```rust
# use amethyst::core::Transform;
# use amethyst::ecs::{System};
# struct FallingObject;
use amethyst::ecs::Join;

struct MakeObjectsFall;

impl System for MakeObjectsFall {
    type SystemData = (.write_component::<Transform>().read_component::<FallingObject>());

    fn run(&mut self, (mut transforms, falling): Self::SystemData) {
        for (transform, _) in (&mut transforms, &falling).join() {
            if transform.translation().y > 0.0 {
                transform.prepend_translation_y(-0.1);
            }
        }
    }
}
```

This system will make all entities with both a `Transform` with a positive y coordinate and a `FallingObject` tag component fall by 0.1 unit per game loop iteration. Note that as the `FallingObject` is only here as a tag to restrict the joining operation, we immediately discard it using the `_` syntax.

Cool! Now that looks like something we'll actually do in our games!

### Getting entities that have some components, but not others

There is a special type of `Storage` in specs called `AntiStorage`.
The not operator (!) turns a Storage into its AntiStorage counterpart, allowing you to iterate over entities that do NOT have this `Component`.
It is used like this:

```rust
# use amethyst::core::Transform;
# use amethyst::ecs::{System};
# struct FallingObject;
use amethyst::ecs::Join;

struct NotFallingObjects;

impl System for NotFallingObjects {
    type SystemData = (.write_component::<Transform>().read_component::<FallingObject>());

    fn run(&mut self, (mut transforms, falling): Self::SystemData) {
        for (mut transform, _) in (&mut transforms, !&falling).join() {
            // If they don't fall, why not make them go up!
            transform.prepend_translation_y(0.1);
        }
    }
}
```

## Manipulating the structure of entities

It may sometimes be interesting to manipulate the structure of entities in a system, such as creating new ones or modifying the component layout of existing ones. This kind of process is done using the `Entities<'a>` system data.

> Requesting `Entities<'a>` does not impact performance, as it contains only immutable resources and therefore [does not block the dispatching](./dispatcher.html).

### Creating new entities in a system

Creating an entity while in the context of a system is very similar to the way one would create an entity using the `World` struct. The only difference is that one needs to provide mutable storages of all the components they plan to add to the entity.

```rust
# use amethyst::core::Transform;
# use amethyst::ecs::{Entities, System};
# struct Enemy;
struct SpawnEnemies {
    counter: u32,
}

impl System for SpawnEnemies {
    type SystemData = (
        .write_component::<Transform>()
        .write_component::<Enemy>()
        Entities<'a>,
    );

    fn run(&mut self, (mut transforms, mut enemies, entities): Self::SystemData) {
        self.counter += 1;
        if self.counter > 200 {
            entities
                .build_entity()
                .with(Transform::default(), &mut transforms)
                .with(Enemy, &mut enemies)
                .build();
            self.counter = 0;
        }
    }
}
```

This system will spawn a new enemy every 200 game loop iterations.

### Removing an entity

Deleting an entity is very easy using `Entities<'a>`.

```rust
# use amethyst::ecs::{Entities, Entity, System};
# struct MySystem {
#   entity: Entity,
# }
# impl System for MySystem {
#   type SystemData = Entities<'a>;
#   fn run(&mut self, entities: Self::SystemData) {
#       let entity = self.entity;
        entities.delete(entity);
#   }
# }
```

### Iterating over components with associated entity

Sometimes, when you iterate over components, you may want to also know what entity you are working with. To do that, you can use the joining operation with `Entities<'a>`.

```rust
# use amethyst::core::Transform;
# use amethyst::ecs::{Entities, System};
# struct FallingObject;
struct MakeObjectsFall;

impl System for MakeObjectsFall {
    type SystemData = (
        Entities<'a>,
        .write_component::<Transform>()
      .read_component::<FallingObject>(),
    );

    fn run(&mut self, (entities, mut transforms, falling): Self::SystemData) {
        for (e, mut transform, _) in (&*entities, &mut transforms, &falling).join() {
            if transform.translation().y > 0.0 {
                transform.prepend_translation_y(-0.1);
            } else {
                entities.delete(e);
            }
        }
    }
}
```

This system does the same thing as the previous `MakeObjectsFall`, but also cleans up falling objects that reached the ground.

### Adding or removing components

You can also insert or remove components from a specific entity.
To do that, you need to get a mutable storage of the component you want to modify, and simply do:

```rust
# use amethyst::ecs::{Entity, System};
# struct MyComponent;
# struct MySystem {
#   entity: Entity,
# }
# impl System for MySystem {
.write_component::<MyComponent>()
#   fn run(&mut self, mut write_storage: Self::SystemData) {
#       let entity = self.entity;
        // Add the component
        write_storage.insert(entity, MyComponent);

        // Remove the component
        write_storage.remove(entity);
#   }
# }
```

Keep in mind that inserting a component on an entity that already has a component of the same type **will overwrite the previous one**.

## Changing states through resources

In a previous section we talked about [`States`][s], and how they are used to organize your game
into different logical sections.
Sometimes we want to trigger a state transition from a system.
For example, if a player dies we might want to remove their entity and signal to the state machine
to push a state that shows a "You Died" screen.

So how can we affect states from systems?
There are a couple of ways, but this section will detail the easiest one: using a [`Resource`][r].

Before that, let's quickly remind ourselves what a resource is:

> A [`Resource`][r] is any type that stores data that you might need for your game AND that is not
> specific to an entity.

The data in a resource is available both to systems and states.
We can use this to our advantage!

Let's say you have the following two states:

- `GameplayState`: State in which the game is running.
- `GameMenuState`: State where the game is paused and we interact with a game menu.

The following example shows how to keep track of which state we are currently in.
This allows us to do a bit of conditional logic in our systems to determine what to do depending on
which state is currently active, and manipulating the states by tracking user actions:

```rust
use amethyst::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CurrentState {
    MainMenu,
    Gameplay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UserAction {
    OpenMenu,
    ResumeGame,
    Quit,
}

impl Default for CurrentState {
    fn default() -> Self {
        CurrentState::Gameplay
    }
}

struct Game {
    user_action: Option<UserAction>,
    current_state: CurrentState,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            user_action: None,
            current_state: CurrentState::default(),
        }
    }
}

struct GameplayState;

impl SimpleState for GameplayState {
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        // If the `Game` resource has been set up to go back to the menu, push
        // the menu state so that we go back.

        let mut game = data.world.write_resource::<Game>();

        if let Some(UserAction::OpenMenu) = game.user_action.take() {
            return Trans::Push(Box::new(GameMenuState));
        }

        Trans::None
    }

    fn on_resume(&mut self, mut data: StateData<'_, GameData>) {
        // mark that the current state is a gameplay state.
        data.world.write_resource::<Game>().current_state = CurrentState::Gameplay;
    }
}

struct GameMenuState;

impl SimpleState for GameMenuState {
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let mut game = data.world.write_resource::<Game>();

        match game.user_action.take() {
            Some(UserAction::ResumeGame) => Trans::Pop,
            Some(UserAction::Quit) => {
                // Note: no need to clean up :)
                Trans::Quit
            }
            _ => Trans::None,
        }
    }

    fn on_resume(&mut self, mut data: StateData<'_, GameData>) {
        // mark that the current state is a main menu state.
        data.world.write_resource::<Game>().current_state = CurrentState::MainMenu;
    }
}
```

Let's say we want the player to be able to press escape to enter the menu.
We modify our input handler to map the `open_menu` action to `Esc`, and we write the following
system:

```rust
# #[derive(Clone, Copy, Debug, PartialEq, Eq)]
# enum CurrentState {
#   MainMenu,
#   Gameplay,
# }
# 
# impl Default for CurrentState {
#   fn default() -> Self {
#       CurrentState::Gameplay
#   }
# }
# 
# #[derive(Clone, Copy, Debug, PartialEq, Eq)]
# enum UserAction {
#   OpenMenu,
#   ResumeGame,
#   Quit,
# }
# 
# struct Game {
#   user_action: Option<UserAction>,
#   current_state: CurrentState,
# }
# 
# impl Default for Game {
#   fn default() -> Self {
#       Game {
#           user_action: None,
#           current_state: CurrentState::default(),
#       }
#   }
# }
# 
use amethyst::{
    ecs::{prelude::*, System},
    input::InputHandler,
    prelude::*,
};

struct MyGameplaySystem;

impl System for MyGameplaySystem {
    type SystemData = (.read_resource::<Game>());

    fn run(&mut self, (input, mut game): Self::SystemData) {
        match game.current_state {
            CurrentState::Gameplay => {
                let open_menu = input.action_is_down("open_menu").unwrap_or(false);

                // Toggle the `open_menu` variable to signal the state to
                // transition.
                if open_menu {
                    game.user_action = Some(UserAction::OpenMenu);
                }
            }
            // do nothing for other states.
            _ => {}
        }
    }
}
```

Now whenever you are playing the game and you press the button associated with the `open_menu`
action, the `GameMenuState` will resume and the `GameplayState` will pause.

## The SystemData trait

While this is rarely useful, it is possible to create custom `SystemData` types.

The `Dispatcher` populates the `SystemData` on every call of the `run` method. To do that, your `SystemData` type must implement the trait `amethyst::ecs::SystemData` in order to have it be valid.

This is rather complicated trait to implement, fortunately Amethyst provides a derive macro for it, that can implement the trait to any struct as long as all its fields are `SystemData`. Most of the time however, you will not even need to implement it at all as you will be using `SystemData` structs provided by the engine.

Please note that tuples of structs implementing `SystemData` are themselves `SystemData`. This is very useful when you need to request multiple `SystemData` at once quickly.

```rust
# extern crate shred;
# #[macro_use]
# extern crate shred_derive;
# 
# use amethyst::{
#   ecs::{System, World},
#   shred::ResourceId,
# };
# 
# struct FooComponent {
#   stuff: f32,
# }
# 
# struct BarComponent {
#   stuff: f32,
# }
# 
# #[derive(SystemData)]
# struct BazSystemData<'a> {
#   field:().read_component::<FooComponent>(),
# }
# 
# impl<'a> BazSystemData<'a> {
#   fn should_process(&self) -> bool {
#       true
#   }
# }
# 
#[derive(SystemData)]
struct MySystemData<'a> {
    foo:().read_component::<FooComponent>(),
    bar: .write_component::<BarComponent>()
    baz: BazSystemData<'a>,
}

struct MyFirstSystem;

impl System for MyFirstSystem {
    type SystemData = MySystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if data.baz.should_process() {
            for (foo, mut bar) in (&data.foo, &mut data.bar).join() {
                bar.stuff += foo.stuff;
            }
        }
    }
}
```

[r]: ./resource.md
[s]: ./state.md
[sb]: https://docs.rs/legion/0.3.1/legion/systems/struct.SystemBuilder.html

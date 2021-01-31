# State

## What is a state?

The word "state" can mean a lot of different things in computer science.
In the case of amethyst, it is used to represent the "game state".

A game state is a *general* and *global* section of the game.

## Example

As an example, let's say you are making a pong game.

- When the user opens up the game, it first loads all the assets and shows a loading screen.
- Then, the main menu shows up, asking you if you want to start a game in single or multiplayer.
- Once you select an option, the game displays the paddles and the ball and starts playing.
- By pressing escape, you can toggle the "pause" menu.
- Once the score limit is reached, a result screen is shown with a button to go back to the main menu.

The game can be divided into different states:

- LoadingState
- MainMenuState
- GameplayState
- PauseState
- ResultState

While you could effectively insert all the game's logic into a single state `GameState`,
dividing it into multiple parts makes it much easier to reason about and maintain.

## State Manager

Amethyst has a built-in state manager, which allows easily switching between different `State`s.
It is based on the concept of a pushdown-automaton, which is a combination of a Stack and a State Machine.

### Stack

The stack concept makes it so you can "push" `State`s on top of each other.

If we take the pong example of earlier, you can push the `PauseState` over the `GameplayState`.

When you want to go out of pause, you pop the `PauseState` out of the stack and you are back into the `GameplayState`, just as you left it.

### State Machine

The concept of State Machine can be pretty complex, but here we will only explain the basics of it.
The State Machine is usually composed of two elements: Transitions and Events.

Transitions are simply the "switching" between two states.

For example, from `LoadingState`, go to state `MainMenuState`.

Amethyst has multiple types of transitions.

- You can Push a `State` over another.
- You can also Switch a `State`, which replaces the current `State` with a new one.

Events are what trigger the transitions. In the case of amethyst, it is the different methods called on the `State`. Continue reading to learn about them.

## Life Cycle

`State`s are only valid for a certain period of time, during which a lot of things can occur.
A `State` contains methods that reflect the most common of those events:

- on\_start: When a `State` is added to the stack, this method is called on it.
- on\_stop: When a `State` is removed from the stack, this method is called on it.
- on\_pause: When a `State` is pushed over the current one, the current one is paused, and this method is called on it.
- on\_resume: When the `State` that was pushed over the current `State` is popped, the current one resumes, and this method is called on the now-current `State`.
- handle\_event: Allows easily handling events, like the window closing or a key being pressed.
- fixed\_update: This method is called on the active `State` at a fixed time interval (1/60th second by default).
- update: This method is called on the active `State` as often as possible by the engine.
- shadow\_update: This method is called as often as possible by the engine on all `State`s which are on the `StateMachines` stack, including the active `State`. Unlike `update`, this does not return a `Trans`.
- shadow\_fixed\_update: This method is called at a fixed time interval (1/60th second by default) on all `State`s which are on the `StateMachines` stack, including the active `State`. Unlike `fixed_update`, this does not return a `Trans`.

If you aren't using `SimpleState` or `EmptyState`, you *must* implement the `update` method to call `data.data.update(&mut data.world)`.

## Game Data

`State`s can have arbitrary data associated with them.
If you need to store data that is tightly coupled to your `State`, the classic way is to put it in the `State`'s struct.

`State`s also have internal data, which is any type T.
In most cases, the two following are the most used: `()` and `GameData`.

`()` means that there is no data associated with this `State`. This is usually used for tests and not for actual games.
`GameData` is the de-facto standard. It is a struct containing a `Dispatcher`. This will be discussed later.

When calling your `State`'s methods, the engine will pass a `StateData` struct which contains both the `World` (which will also be discussed later) and the Game Data type that you chose.

## Code

Yes! It's finally time to get some code in here!

Here will be a small code snippet that shows the basics of `State`'s usage.
For more advanced examples, see the following pong tutorial.

### Creating a State

```rust
use amethyst::prelude::*;

struct GameplayState {
    /// The `State`-local data. Usually you will not have anything.
    /// In this case, we have the number of players here.
    player_count: u8,
}

impl SimpleState for GameplayState {
    fn on_start(&mut self, _data: StateData<'_, GameData>) {
        println!("Number of players: {}", self.player_count);
    }
}
```

That's a lot of code, indeed!

We first declare the `State`'s struct `GameplayState`.

In this case, we give it some data: `player_count`, a byte.

Then, we implement the `SimpleState` trait for our `GameplayState`.
`SimpleState` is a shorthand for `State<GameData, StateEvent>` where `GameData` is the shared data between states and `StateEvent` is
an enum of events that can be received by the `State` on its `handle_event` method.

### Switching State

Now, if we want to change to a second state, how do we do it?

Well, we'll need to use one of the methods that return the `Trans` type.

Those are:

- `handle_event`
- `fixed_update`
- `update`

Let's use `handle_event` to go to the `PausedState` and come back by pressing the "Escape" key.

```rust
use amethyst::{
    input::{is_key_down, VirtualKeyCode},
    prelude::*,
};

struct GameplayState;

struct PausedState;

// This time around, we are using () instead of GameData, because we don't have any `System`s that need to be updated.
// (They are covered in the dedicated section of the book.)
// Instead of writing `State<(), StateEvent>`, we can instead use `EmptyState`.
impl EmptyState for GameplayState {
    fn handle_event(&mut self, _data: StateData<()>, event: StateEvent) -> EmptyTrans {
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                // Pause the game by going to the `PausedState`.
                return Trans::Push(Box::new(PausedState));
            }
        }

        // Escape isn't pressed, so we stay in this `State`.
        Trans::None
    }
}

impl EmptyState for PausedState {
    fn handle_event(&mut self, _data: StateData<()>, event: StateEvent) -> EmptyTrans {
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                // Go back to the `GameplayState`.
                return Trans::Pop;
            }
        }

        // Escape isn't pressed, so we stay in this `State`.
        Trans::None
    }
}
```

### Event Handling

As you already saw, we can handle events from the `handle_event` method.
But what is this weird `StateEvent` all about?

Well, it is simply an enum. It regroups multiple types of events that are emitted throughout the engine by default.
To change the set of events that the state receives, you create a new event enum and derive `EventReader` for that type.

```rust
// These imports are required for the #[derive(EventReader)] code to build
use amethyst::{
    core::{
        ecs::World,
        shrev::{EventChannel, ReaderId},
        EventReader,
    },
    input::{is_key_down, VirtualKeyCode},
    prelude::*,
    ui::UiEvent,
    winit::Event,
};

#[derive(Clone, Debug)]
pub struct AppEvent {
    data: i32,
}

#[derive(Debug, EventReader, Clone)]
#[reader(MyEventReader)]
pub enum MyEvent {
    Window(Event),
    Ui(UiEvent),
    App(AppEvent),
}

struct GameplayState;

impl State<(), MyEvent> for GameplayState {
    fn handle_event(&mut self, _data: StateData<()>, event: MyEvent) -> Trans<(), MyEvent> {
        match event {
            MyEvent::Window(_) => {} // Events related to the window and inputs.
            MyEvent::Ui(_) => {}     // Ui event. Button presses, mouse hover, etc...
            MyEvent::App(ev) => println!("Got an app event: {:?}", ev),
        };

        Trans::None
    }
}
```

To make `Application` aware of the change to which events to send to the state, you also need to supply both the
event type, and the `EventReader` type (the name you give in the `#[reader(SomeReader)]` derive attribute) when
the `Application` is created. This is done by replacing `Application::build` (or `Application::new`) with
`CoreApplication::<_, MyEvent, MyEventReader>::build()` (or `CoreApplication::<_, MyEvent, MyEventReader>::new()`).

*Note: Events are gathered from `EventChannel`s. `EventChannel`s are covered in the dedicated book section.*

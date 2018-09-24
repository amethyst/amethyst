# State

## What is a state?
The word "state" can mean a lot of different things in computer science.
In the case of amethyst, it is used to represent the "game stage".

A game stage is a *general* and *global* section of the game.

## Example

Let's go with an example.
You are making a pong game.

* When the user opens up the game, it first loads all the assets and shows a loading screen.
* Then, the main menu shows up, asking you if you want to start a game in single or multiplayer.
* Once you select an option, the game displays the paddles and the ball and starts playing.
* By pressing escape, you can toggle the "pause" menu.
* Once the score limit is reached, a result screen is shown with a button to go back to the main menu.

All of these can be divided in different states:
* LoadingState
* MainMenuState
* GameplayState
* PauseState
* ResultState

While you could effectively insert all the game's logic into a single state `GameState`,
dividing it in multiple parts makes it much easier to reason about and maintain.

## State Manager

Amethyst has a built-in state manager, which allows easily switching between different `State`s.
It is based on the concept of a pushdown-automaton, which is a combination of a Stack and a State Machine.

### Stack

The stack concept makes it so you can "push" `State`s on top of each other.

If we take the pong example of earlier, you can push the `PauseState` over the `GameplayState`.

When you want to go out of pause, you pop the `PauseState` out of the stack and you are back into the `GameplayState`, just as you left it.

### State Machine

The concept of State Machine can be pretty complex, but here I will only explain the basics of it.
The State Machine is usually composed of two elements: Transitions and Events.

Transitions are simply the "switching" between two states.

For example, from `LoadingState`, go to state `MainMenuState`.

Amethyst has multiple types of transitions.
* You can Push a `State` over another.
* You can also Switch a `State`, which replaces the current `State` by a new one.

Events are what trigger the transitions. In the case of amethyst, it is the different methods called on the `State`. Continue reading to learn about them.

## Life Cycle

`State` are only valid for a certain period of time, during which a lot of things can occur.
A `State` contains methods that reflect the most commons of those events:
* on_start: When the state is added to the stack, this method is called.
* on_stop: When it is removed from the stack, this method is called.
* on_pause: When a `State` is pushed over the current one, the current one is paused.
* on_resume: When the `State` that was pushed over the current `State` is popped, the current one resumes.
* handle_event: Allows easily handling events, like the window closing or a key being pressed.
* fixed_update: This method is called at a fixed time interval (default 1/60th second).
* update: This method is called as often as possible by the engine.

## Game Data

`State`s can have arbitrary data associated with them.
If you need to store data that is tightly coupled to your `State`, the classic way is to put it in the `State`'s struct.

`State`s also have internal data, which is any type T.
In most cases, the two following are the most used: `()` and `GameData`.

`()` means that there is no data associated with this `State`. This is usually used for tests and not for actual games.
`GameData` is the de-facto standard. It is a struct containing a `Dispatcher` (which will be discussed later).

When calling your `State`'s methods, the engine will pass a `StateData` struct which contains both the `World` (which will also be discussed later) and the Game Data type that you chose.

## Code

Yes! It's finally time to get some code in here!

Here will just be a small code snippet that shows the basics of `State`'s usage.
For more advanced examples, see the following pong tutorial.

### Creating a State

```rust,no_run,noplaypen
extern crate amethyst;
use amethyst::prelude::*;

struct GameplayState {
    /// The `State`-local data. Usually you will not have anything.
    /// In this case, we have the number of players here.
    player_count: u8,
}

impl<'a,'b> SimpleState<'a,'b> for GameplayState {
    fn on_start(&mut self, _data: StateData<GameData>) {
        println!("Number of players: {}", self.player_count);
    }
}
```

That's a lot of code, indeed!

We first declare the `State`'s struct `GameplayState`.

In this case, we give it some data: player_count as byte.

Then, we implement the `SimpleState` trait for our `GameplayState`.
`SimpleState` is a shorthand for `State<GameData<'a, 'b>, ()>` where `GameData` is the internal shared data between states.

### Switching State

Now, if we want to change to a second state, how do we do it?

Well, we'll need to use one of the methods that return the `Trans` type.

Those are:
* handle_event
* fixed_update
* update

Let's use handle_event to go to the `PausedState` and come back by pressing the "Escape" key.

```rust,no_run,noplaypen
extern crate amethyst;
use amethyst::prelude::*;
use amethyst::renderer::VirtualKeyCode;
use amethyst::input::is_key_down;

struct GameplayState;
struct PausedState;

// This time around, we are using () instead of GameData, because we don't have any `System`s that need to be updated.
// (They are covered in the dedicated section of the book.)
// Instead of writing `State<(), ()>`, we can instead use `EmptyState`.
impl EmptyState for GameplayState {
    fn handle_event(&mut self, _data: StateData<()>, event: StateEvent<()>) -> EmptyTrans {
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
    fn handle_event(&mut self, _data: StateData<()>, event: StateEvent<()>) -> EmptyTrans {
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
But what is this weird `StateEvent<()>` all about?

Well, it is simply an enum. It regroups multiple types of events that are emitted throughout the engine.
The generic parameter `()` indicates that we don't have any custom event type that we care about.
If you were to replace `()` by `MyEventType`, then you could react to them from your state like so:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::renderer::VirtualKeyCode;
# use amethyst::input::is_key_down;

#[derive(Debug)]
struct MyEvent {
    data: i32,
}

struct GameplayState;

impl State<(), MyEvent> for GameplayState {
    fn handle_event(&mut self, _data: StateData<()>, event: StateEvent<MyEvent>) -> Trans<(), MyEvent> {
        match event {
            StateEvent::Window(_) => {}, // Events related to the window and inputs.
            StateEvent::Ui(_) => {}, // Ui event. Button presses, mouse hover, etc...
            StateEvent::Custom(ev) => println!("Got a custom event: {:?}", ev),
        }
        
        Trans::None
    }
}
```

*Note: Events are gathered from `EventChannel`s. If you use a custom event, you need to write events to the global `EventChannel<MyEvent>`. `EventChannel`s are covered in the dedicated book section.*

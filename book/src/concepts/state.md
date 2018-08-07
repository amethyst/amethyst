# State

## What is a state?
The word "state" can mean at lot of different things in computer science.
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


**IMPORTANT: In order to have the game working, you NEED to implement the `update` method and have it call `data.data.update(&mut data.world)`. This is an implementation detail and will no longer be necessary in future release 0.9 of Amethyst. For more information, read the "More" section at the end of the world chapter [here](./world.md#more).**

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

```rust
extern crate amethyst;
use amethyst::{GameData, State, StateData};

struct GameplayState {
    /// The `State`-local data. Usually you will not have anything.
    /// In this case, we have the number of players here.
    player_count: u8,
}

impl<'a,'b> State<GameData<'a,'b>> for GameplayState {
    fn on_start(&mut self, _data: StateData<GameData<'a,'b>>) {
        println!("Number of players: {}", self.player_count);
    }
}
```

That's a lot of code, indeed!

We first declare the `State`'s struct.

In this case, we give it some data: player_count as byte.

Then, we implement the `State` trait for our `GameplayState`, and we specify that we want to have the `GameData` internal data.

### Switching State

Now, if we want to change to a second state, how do we do it?

Well, we'll need to use one of the methods that return the `Trans` type.

Those are:
* handle_event
* fixed_update
* update

Let's use handle_event to go to the `PausedState` and come back by pressing the "Escape" key.

```rust
extern crate amethyst;
use amethyst::{State, StateData, Trans};
use amethyst::renderer::{Event, VirtualKeyCode};

struct GameplayState;
struct PausedState;

// This time around, we are using () instead of GameData, because we don't have any `System`s.
// `System`s will be covered later in this book.
impl State<()> for GameplayState {
    fn handle_event(&mut self, _data: StateData<()>, event: Event) -> Trans<()> {
        if is_key_down(&event, VirtualKeyCode::Escape) {
            // Pause the game by going to the `PausedState`.
            return Trans::Push(Box::new(PausedState));
        }
        // Escape isn't pressed, so we stay in this `State`.
        Trans::None
    }
}

impl State<()> for PausedState {
    fn handle_event(&mut self, _data: StateData<()>, event: Event) -> Trans<()> {
        if is_key_down(&event, VirtualKeyCode::Escape) {
            // Go back to the `GameplayState`.
            return Trans::Pop;
        }
        // Escape isn't pressed, so we stay in this `State`.
        Trans::None
    }
}

```

# Setting up a blank application

Currently amethyst development is moving faster than the amethyst_tools app, so
we need to manually setup a blank project.

## Instructions

 1. Run `cargo new --bin pong` to create a new rust app called pong.

 2. Add amethyst to your dependencies.

 ```toml
 ...
 [dependencies]
 amethyst = "*"
 ...
 ```

 3. Put the following in your `src/main.rs`

 ```rust
 extern crate amethyst;
 
 use amethyst::{Application, State, Trans};
 use amethyst::asset_manager::AssetManager;
 use amethyst::renderer::{Pipeline};
 use amethyst::gfx_device::DisplayConfig;
 use amethyst::config::Element;
 use amethyst::ecs::{World};
 
 struct Pong;
 
 impl State for Pong {
     fn on_start(&mut self, _world: &mut World, _assets: &mut AssetManager, _pipe: &mut Pipeline) {
         println!("Game started!");
     }
 
     fn update(&mut self, _world: &mut World, _assets: &mut AssetManager, _pipe: &mut Pipeline) -> Trans{
         println!("Hello from Amethyst!");
         Trans::Quit
     }
 
     fn on_stop(&mut self, _world: &mut World, _assets: &mut AssetManager, _pipe: &mut Pipeline) {
         println!("Game stopped!");
     }
 }
 
 fn main() {
     let path = format!("{}/resources/config.yml", env!("CARGO_MANIFEST_DIR"));
     let cfg = DisplayConfig::from_file(path).unwrap();
     let mut game = Application::build(Pong, cfg).done();
     game.run();
 }
 ```

 4. Add the following to `resources/config.yaml`

 ```yaml
 dimensions: null
 fullscreen: false
 max_dimensions: null
 min_dimensions: null
 multisampling: 1
 title: "Pong example"
 visibility: true
 vsync: true
 ```

 5. Now type `cargo run` to make sure everything is working. You should see

 ```
 Game started!
 Hello from amethyst!
 Game stopped!
 ```

 If not, make sure you copied all the above text exactly. We're now going to go
 through the code, explaining how it works.

## Explanation

```rust
extern crate amethyst;

use amethyst::{Application, State, Trans};
use amethyst::asset_manager::AssetManager;
use amethyst::renderer::{Pipeline};
use amethyst::gfx_device::DisplayConfig;
use amethyst::config::Element;
use amethyst::ecs::{World};
```

This first part just loads our dependencies into our namespace. It isn't very
interesting on its own.

```rust
struct Pong;

impl State for Pong {
```

Here we describe how our application works. An amethyst app is a stack of
struct objects implementing the `State` trate. These objects describe different 
modes that the app can be in. When a state knows it has finished, it can pop
itself off the stack without knowing which state it needs to transition to. We
will have only a single state - which we name `Pong`.

```rust
fn on_start(&mut self, 
            _world: &mut World, 
            _assets: &mut AssetManager, 
            _pipe: &mut Pipeline) {
    println!("Game started!");
}
```
This method is run when a state is first loaded onto the stack, and can be used
to do any setup and initialization needed. I'm starting the parameter names
with an underscore `_` so that the compiler doesn't complain I'm not using
them.

```rust
fn update(&mut self, 
          _world: &mut World, 
          _assets: &mut AssetManager, 
          _pipe: &mut Pipeline) -> Trans {
    println!("Hello from Amethyst!");
    Trans::Quit
}
```
This method is run as fast as possible in the main event loop, and is
responsible for updating the game world. It returns a `Trans` object, that can
be used to transition to another state (push it onto the stack), pop itself, 
or quit. We will normally not use this method, and rely on the *ecs* system
(discussed in future steps) to move forward the simulation. Here we tell the
app to immediately quit.

```rust
fn on_stop(&mut self, 
           _world: &mut World, 
           _assets: &mut AssetManager, 
           _pipe: &mut Pipeline) {
    println!("Game stopped!");
}
```
This method is run when a state is popped off the stack, and can be used to
clean up.

We haven't implemented all the `State` methods, see the module documentation
for an exhaustive list. The names are helpfully descriptive.

```rust
fn main() {
    let path = format!("{}/resources/config.yml", env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Pong, cfg).done();
    game.run();
}
```
Here we load the render settings from `resources/config.yml` and create our
app, with the `Pong` state as the initial state.

Now, we're ready to start hacking ;).

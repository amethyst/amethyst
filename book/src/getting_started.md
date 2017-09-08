# Getting Started

These instructions will help you set up a working Amethyst game development
environment. Then we'll test this environment by compiling a simple "Hello,
World" application.

Amethyst officially supports the following Rust targets:

### Windows
x86_64-pc-windows-msvc

i686-pc-windows-msvc

x86_64-pc-windows-gnu

i686-pc-windows-gnu

### Mac OS
x86_64-apple-darwin

i686-apple-darwin

### Linux
x86_64-unknown-linux-gnu

i686-unknown-linux-gnu

### Other

Other desktop PC targets might work but we are not currently officially
supporting them.


## Setting Up

> Note: This guide assumes you have nightly Rust and Cargo installed, and also
> have a working Internet connection.
> Please take care of these prerequisites first before proceeding.
> See [rustup][ru] for handling multiple rust toolchains.

[ru]: https://www.rustup.rs/

## Rust project basics

**If you consider yourself a rust veteran you can probably skip this section,
just create a binary project and add a dependency for amethyst from crates.io.**

Once you have that setup you'll need to make a new cargo binary project for your
game to be written in.  You can do this with the following command

```
$ cargo init --bin my_cool_game
```

Where my_cool_game is the name of your project.

Next, we need to pull in the `amethyst` crate as a dependency. To do this visit
[our crates.io page](https://crates.io/crates/amethyst) and grab the latest version number from that
page.  Now go to the Cargo.toml file inside your project directory and add

```toml
[dependencies]
amethyst = "x.x"
```

where x.x are the first two sections of the version number you grabbed from
Amethyst.  This way you can receive non-breaking patches from us with
`cargo update` but you won't receive breaking changes unless you change your
version number in Cargo.toml.
This is because amethyst follows [semver](http://semver.org/spec/v2.0.0.html).

## Amethyst specific setup

Next you'll need a resources folder that will contain the data for your game.
You can call this anything, but our convention is just `resources`.  Stick it in
your project directory nex to Cargo.toml.

### config.ron

  The first file you'll want to create in here is `config.ron` this file is
written in the [RON](https://github.com/ron-rs/ron) format.  The format
of these files is similar to Rust, so it shouldn't be too alien.
Its contents will look like this:

```
(
  title: "My cool game",
  dimensions: None,
  max_dimensions: None,
  min_dimensions: None,
  fullscreen: false,
  multisampling: 1,
  visibility: true,
  vsync: true,
)
```

We'll go through each line.

`title: "My cool game",` This sets the title of the game window.

`dimensions: None,` This uses the Rust [Option](https://doc.rust-lang.org/std/option/enum.Option.html)
type. This field if set will define the size of the window. Right now it's not
 set. If we wanted to set it we'd use `Some((1920, 1080))` to get a 1920x1080
 window.

`max_dimensions: None,` This is similar to the dimensions field, but this sets a
maximum size.

`min_dimensions: None,` This is similar to the dimensions field, but this sets a
minimum size.

`fullscreen: false,` If this is true then this window will be fullscreen.

`multisampling: 1,` This defines the level of [MSAA anti-aliasing](https://en.wikipedia.org/wiki/Multisample_anti-aliasing).

`visibility: true,` If this is false the window will not be immediately visible
on startup.

`vsync: true,` If this is true then the game will use [vertical synchronization](https://en.wikipedia.org/wiki/Screen_tearing#V-sync).

This file can be used to make a `DisplayConfig` which we can then use to make a
window.

Now we're ready to start coding.

### Creating our first state

  Now that we have our DisplayConfig in place we're going to get an
Amethyst Application setup.  Open main.rs and at the top of your file
write

```rust,ignore
extern crate amethyst;
```

Then under that write

```rust,ignore
use amethyst::prelude::*;
```

This will get you setup with the most important amethyst types.
In between `use amethyst...` and `fn main() {` start creating a state.
To do this first you'll need to make a new struct for your state, let's
give it a name.

```rust,ignore
struct MyCoolGame;
```

Now we need to `impl State for MyCoolGame` do this with

```rust,ignore
impl State for MyCoolGame {
  fn on_start(&mut self, eng: &mut Engine) {

  }

  fn on_stop(&mut self, eng: &mut Engine) {

  }

  fn on_pause(&mut self, eng: &mut Engine) {

  }

  fn on_resume(&mut self, eng: &mut Engine) {

  }

  fn handle_event(&mut self, eng: &mut Engine, event: Event) -> Trans {

  }

  fn fixed_update(&mut self, eng: &mut Engine) -> Trans {

  }

  fn update(&mut self, eng: &mut Engine) -> Trans {

  }
}
```

Phew!  There's a lot there.  Not all of these are necessary, so you can
remove the ones you don't end up using.  Let's go over what these do.

- `on_start` Does what it says on the tin.  This is the first function a
  state executes.
- `on_stop` Also does what it says on the tin.  This is the last
  function a state executes.
- `on_pause` This is executed when a new game state is pushed onto the
  stack, replacing this one as the top state.
  // FIXME: There actually isn't a way to push more than one state onto the
stack right now.
- `on_resume` This is executed when the state above this one on the
  stack is popped off of it, making this state the currently active one
again.
- `handle_event` Executed whenever an event is available from the game
  engine.
- `fixed_update` Executed repeatedly at stable intervals.
- `update` Executed on every frame.

Now that we have a structure that implements the State trait we can
initialize an Application.  First we'll want to load our DisplayConfig.
To do this write this inside `fn main()`

```rust,ignore
let config = DisplayConfig::load("resources/config.ron");
```

Now we can build our Application with

```rust,ignore
let mut game = Application::build(MyCoolGame)
  .expect("Initializing builder failed")
  .with_renderer(Pipeline::forward::<PosNormTex>(), Some(config))
  .expect("Failed to add renderer")
  .build()
  .expect("Failed to build Application");

game.run();
```

If all goes well here you should see a simple black window come up when
you run this application.

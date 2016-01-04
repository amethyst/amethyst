# Getting Started

These instructions will help you set up a working Amethyst development
environment. Then we'll test this environment by compiling a simple "Hello,
World" application.

Amethyst should compile hassle-free on any platform properly supported by the
Rust compiler. Here are the system requirements:

* Minimum:
  * CPU: 1GHz, the more cores the better
  * RAM: 512 MiB
  * HDD: 30 MiB free disk space
  * OS: Windows Vista and newer, Linux, BSD, Mac OS X
  * Rust: Nightly (1.6.0 or newer)
* Renderer Backends:
  * OpenGL 4.0

## Installation

> Note: This guide assumes you have nightly Rust and Cargo installed, and also a
> working Internet connection. Please take care of these first before
> proceeding.

Create a new Rust executable project with `cargo new --bin crate_name` and add
the following lines to your "Cargo.toml":

```toml
[dependencies]
amethyst = "0.1.0"
```

### Manual Installation

If you don't want to get Amethyst from crates.io, you can download and compile
Amethyst by hand:

```
git clone https://github.com/ebkalderon/amethyst.git
cd amethyst
cargo build
```

Then, in your crate's "Cargo.toml", specify the location of the `amethyst`
library yourself:

```toml
[dependencies.amethyst]
lib = "../path/to/amethyst/lib.rs"

TODO: Need to verify accuracy!
```

## Hello, World!

Now, we will commence our journey in writing Amethyst applications with the
obligatory "Hello, World" program! In your crate's "main.rs" file, type out or
copy and paste the following code:

```rust
extern crate amethyst;

use amethyst::engine::{Application, Duration, Game, State};

struct HelloWorld;

impl State for HelloWorld {
    fn new() -> HelloWorld {
        HelloWorld
    }

    fn update(&mut self, game: &Game, delta: Duration) {
        println!("Hello from Amethyst!");
        game.quit();
    }
}

fn main() {
    let mut game = Application::new(HelloWorld);
    game.run();
}
```

Then, compile and run the code with:

```
cargo run
```

If you see the output "Hello from Amethyst!" print to your terminal, then
congratulations! You have successfully written your first Amethyst application!

## What did I just do?

Whoa, that went by too fast. Let's slow down a bit and break down this example
program into bite-sized chunks. What we did is create a basic Rust crate that
hooks into the Amethyst game engine, prints some text, and then signals to the
engine to quit.

Amethyst operates like a state machine. Each Amethyst application has one or
more states, each of which roughly correspond to a "screen" seen in a game (e.g.
main menu, loading screen, in-game, inventory menu, credits). You control when
Amethyst should switch states and what happens every time you do. In our "Hello,
World" example, the logic flows like this:

1. Initialize the program.
2. Create engine with `HelloWorld` as the initial state.
3. Start the engine.
   1. Enter the `HelloWorld` state.
   2. On the first frame, print "Hello from Amethyst!", and signal to the engine
      to quit.
   3. Leave the `HelloWorld` State.
4. Shut the engine down.
5. Quit the program.

## Can I make a game yet?

This program isn't very useful on its own, but if everything compiles correctly,
then your environment is set up correctly and you can start working on the fun
stuff! Follow along in [the next section][sa] to see how I implement a simple
Pong clone in Amethyst.

[sa]: ./simple_application.html

# Hello, World!

Now, we will commence our journey in writing Amethyst applications with the
obligatory "Hello, World" program! In your crate's "main.rs" file, type out or
copy and paste the following code:

```rust
extern crate amethyst;

use amethyst::prelude::*;

struct HelloWorld;

impl State for HelloWorld {
    fn on_start(&mut self, _: &mut Engine) {
        println!("Game started!");
    }

    fn update(&mut self, _: &mut Engine) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self, _: &mut Engine) {
        println!("Game stopped!");
    }
}

fn main() {
    let cfg = Config::default();
    let mut game = Application::build(HelloWorld, cfg).finish().expect("Fatal");
    game.run();
}
```

Then, compile and run the code with `cargo run`,
or `amethyst run` if you have the [CLI tool installed][ct].

[ct]: ./getting_started/automatic_setup.html

You should see the following output print to your terminal:

```
Game started!
Hello from Amethyst!
Game stopped!
```

If instead you see `error: use of unstable library feature` then make sure
you're using the [nightly release][nr] of Rust. You can use [rustup][ru] to
install stable and nightly Rust side-by-side. 

[nr]: https://doc.rust-lang.org/book/release-channels.html
[ru]: https://www.rustup.rs

Congratulations! You have successfully written your first Amethyst application.

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

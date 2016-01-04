# Amethyst

[![Build Status][s1]][tc] [![MIT License][s2]][ml] [![Join the chat][s3]][gc]

[s1]: https://travis-ci.org/ebkalderon/amethyst.svg?branch=master
[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[s3]: https://badges.gitter.im/ebkalderon/amethyst.svg

[ml]: https://github.com/ebkalderon/amethyst/blob/master/COPYING
[tc]: https://travis-ci.org/ebkalderon/amethyst/
[gc]: https://gitter.im/ebkalderon/amethyst?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge

Experimental data-oriented game engine written in Rust. This project is a *work
in progress* and very incomplete; pardon the dust!

# Hello, World!

Create a new Cargo binary project with `cargo new --bin crate_name` and add the
following lines to your "Cargo.toml":

```toml
[dependencies.amethyst]
git = "https://github.com/ebkalderon/amethyst.git"
branch = "master"
```

Then, in your "main.rs", type or copy and paste the following code:

```rust
extern crate amethyst;

use amethyst::engine::{Application, Duration, State};

struct HelloWorld;

impl State for HelloWorld {
    fn new() -> HelloWorld {
        HelloWorld
    }

    fn on_start(&mut self) {
        println!("Begin!");
    }

    fn update(&mut self, _delta: Duration) {
        println!("Hello from Amethyst!");
    }

    fn on_stop(&mut self) {
        println!("End!");
    }
}

fn main() {
    let mut game = Application::new(HelloWorld);
    game.run();
}
```

Compile and run the example with `cargo run`. You should see "Hello from
Amethyst!" print out to the terminal and abruptly exit.

# Documentation

Read the associated [online book][bk] for a comprehensive tutorial to using
Amethyst. There is also an online crate-level [API reference][ar].

[bk]: http://ebkalderon.github.io/amethyst/
[ar]: http://ebkalderon.github.io/amethyst/doc/amethyst/index.html

## Building Locally

You can build the book locally with:

```
cargo install mdbook
mdbook build book
```

The text can be found in `book/html/index.html`. To generate the API
documentation locally, do:

```
cargo doc
```

The API reference can be found in `target/doc/amethyst/index.html`.

# Contributing

Amethyst is an open-source project that values community contribution. Pull
requests are welcome!

We assume you have granted non-exclusive right to your source code under the
[MIT license][ml] and you have processed your code with `rustfmt` prior to
submission. If you want to be known as an author, please add your name to the
AUTHORS.md file in the pull request.

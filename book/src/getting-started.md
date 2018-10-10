# Getting started

## Setting up Rust

We recommend using [rustup][ru] to easily install the latest stable version of rust.
Instructions should be on screen once rustup is downloaded.

[ru]: https://rustup.rs

## Required dependencies

If you are on Linux, you'll need to install `libasound2-dev`, `libx11-xcb-dev` and `libssl-dev`.

See the readme on github for distribution specific details.

## Setting up Amethyst

You can either use the [Amethyst CLI][cl] or cargo to set up your project.

### Amethyst CLI (Easiest)
If you wish to use the Amethyst cli tool, you can install it like so

```norun
cargo install amethyst_tools
```

and then run

```norun
amethyst new <game-name>
```

you should get `Cargo.toml`, `src/main.rs` and `resources/display_config.ron`.

### Cargo (Manual)

In case you're doing this with `cargo`, here's what you need to do:

* Add `amethyst` as dependency in your `Cargo.toml`.
* Create a `resources` folder and put a `display_config.ron` in it.
* (Optional) Copy the code from one of amethyst's examples.


[cl]: https://github.com/amethyst/tools

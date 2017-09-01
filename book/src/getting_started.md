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

Next you'll want to add a dependency to amethyst.  To do this visit
[https://crates.io/crates/amethyst] and grab the latest version number from that
page.  Now go to the Cargo.toml file inside your project directory and add
```toml
[dependencies]
amethyst = "x.x"
```
where x.x are the first two sections of the version number you grabbed from
amethyst.  This way you can receive non-breaking patches from us with
`cargo update` but you won't receive breaking changes unless you change your
version number in Cargo.toml.
This is because amethyst follows [semver](http://semver.org/spec/v2.0.0.html).

## Amethyst specific setup

Next you'll need a resources folder that will contain the data for your game.
You can call this anything, but our convention is just `resources`.  Stick it in
your project directory next to Cargo.toml.

### config.ron

  The first file you'll want to create in here is `config.ron` this file is
written in the [RON](https://github.com/ron-rs/ron) format.  It's contents will
look like this:

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

This file is used to generate a `DisplayConfig` which we can then use to make a
window.

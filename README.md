<img align="left" width="64px" src="book/images/amethyst_thumb.png" />

# Amethyst

[![Build Status][s1]][tc] [![Crates.io][s2]][ci] [![MIT/Apache][s3]][li] [![Join the chat][s4]][gc] [![Join us on Discord][s5]][di] ![Lines of Code][s6]

[s1]: https://travis-ci.org/amethyst/amethyst.svg?branch=master
[s2]: https://img.shields.io/crates/v/amethyst.svg
[s3]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[s4]: https://badges.gitter.im/amethyst/general.svg
[s5]: https://img.shields.io/discord/425678876929163284.svg?logo=discord
[s6]: https://tokei.rs/b1/github/amethyst/amethyst?category=code

[tc]: https://travis-ci.org/amethyst/amethyst/
[ci]: https://crates.io/crates/amethyst/
[li]: COPYING
[gc]: https://gitter.im/orgs/amethyst/rooms
[di]: https://discord.gg/GnP5Whs

## What is amethyst?!

Amethyst is a game engine aiming to be fast and as configurable as possible.

## Principles

Those principles are what make amethyst unique and a leader in the world of game engines.

* Massively parallel architecture.
* Powered by a correct [Entity Component System][ecs] model.
* Rapid prototyping with [RON] (Json-like) files for prefabs and an abstract scripting API.
* Strong focus on encouraging reusability and clean interfaces.

[ecs]: https://en.wikipedia.org/wiki/Entity–component–system
[RON]: https://github.com/ron-rs/ron

## Why amethyst? Why not some other engine?

### Extreme Multithreading
While other game engines are all really good (Unity, Unreal, JMonkeyEngine, Godot, LibGdx, ...), they all lack engine-level multithreading support.

This allows game built with amethyst to use up to 99.9% of your processing power to make it run as smooth as possible.

Using the [ecs] architecture, the code of games can be cleanly divided between data and behaviour, making it easy to understand what is going on,
even if the game is running on a massive 64 threads processor.

### Clean

By design, the amethyst engine encourages you to write clean and reusable code for your behaviours and data structures, allowing engine users to easily
share useful components, thus reducing development time and cost.

### Community

While we may not be feature-packed (yet!), we all strongly believe that the community-oriented side of amethyst will make it thrive forward!

## Features

Please visit the [features page][feat] for a list of features Amethyst provides.

[feat]: docs/FEATURES.md

## Documentation

[![develop docs][adb1]][ad1] [![master docs][adb2]][ad2]

[adb1]: https://img.shields.io/badge/docs-develop-blue.svg
[adb2]: https://img.shields.io/badge/docs-master-blue.svg

[ad1]: https://www.amethyst.rs/doc/develop.html
[ad2]: https://www.amethyst.rs/doc/master.html

## Usage

Amethyst is written in rust and thus is considered by many that are learning the basics of rust as hard.
Don't be afraid of challenges! Rust is a wonderful language once you get to know it.
We made a lot of [documentation][bk] that will teach you everything you need to use amethyst like a pro!

[bk]: https://www.amethyst.rs/book/master/

## Getting started

To compile any of the examples run:
```
$ cargo run --example name_of_example
```
All available examples are listed under [examples][ex].

Our most advanced example is currently called pong. It is a pong game, as you guessed it.
```
$ cargo run --example pong
```

There is quite a few prototype games that were made with amethyst. A list will be available soon.
While we create this list, feel free to join our discord and ask about which projects are currently being made with amethyst.

For a full-blown "Hello World" tutorial check out the [Getting Started][gs] chapter
in the book.

[ex]: examples/
[gs]: https://www.amethyst.rs/book/master/getting_started.html

## Dependencies

If you are compiling on Linux make sure to install the following dependencies:

### Ubuntu

```
$ sudo apt install libasound2-dev libx11-xcb-dev
```

### Fedora

```
$ sudo dnf install alsa-lib-devel
```

## Building Documentation

You can build the book locally with:

```
$ cargo install mdbook
$ mdbook build book
```

The text can be found in `book/html/index.html`. To generate the API
documentation locally, do:

```
$ cargo doc
```

The API reference can be found in `target/doc/amethyst/index.html`.

## Questions / Help

We do not support anything other than the most recent Rust stable release. Use nightly and beta channels with this project at your own risk.

Please check out the [FAQ][faq] before asking.

If you have a question, just ask on [Gitter][gt] or [Discord][di] (most active) and we'll help you.

Other places you may want to check out are [r/rust_gamedev][rg] and [#rust-gamedev IRC][irc].

[faq]: https://github.com/amethyst/amethyst/wiki/Frequently-Asked-Questions
[gt]: https://gitter.im/amethyst/general
[di]: https://discord.gg/GnP5Whs
[rg]: https://www.reddit.com/r/rust_gamedev/
[irc]: https://botbot.me/mozilla/rust-gamedev/

## Contributing

We are a community project that welcomes contribution from anyone. If you're
interested in helping out, please read the [CONTRIBUTING.md][cm] file before
getting started. Don't know what to hack on? Check our [active projects][pr], or search though [our issue tracker][it].

[cm]: docs/CONTRIBUTING.md
[pr]: https://github.com/amethyst/amethyst/projects
[it]: https://github.com/amethyst/amethyst/issues

We have a [good first issue][gfi] category that groups all issues or feature request that can be made without having an extensive knowledge of rust or amethyst.
Working on those issues is a good, if not the best way to learn.

[gfi]: https://github.com/amethyst/amethyst/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22

## License

Amethyst is free and open source software distributed under the terms of both
the [MIT License][lm] and the [Apache License 2.0][la].

[lm]: docs/LICENSE-MIT
[la]: docs/LICENSE-APACHE

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

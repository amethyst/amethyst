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

**Warning: The book and the documentation are missing content. Amethyst is undergoing a lot of changes at the moment.**

This project is a *work in progress* and is very incomplete; pardon the dust!

## Goals

* massively parallel architecture
* powered by a correct [Entity Component System][ecs] model
* rapid prototyping with [RON] files for prefabs and an abstract scripting API

[ecs]: https://en.wikipedia.org/wiki/Entity–component–system
[RON]: https://github.com/ron-rs/ron

## Features

Please visit the [features page][feat] for a list of features Amethyst provides and will provide.

[feat]: docs/FEATURES.md

## Documentation

[![develop docs][adb1]][ad1] [![master docs][adb2]][ad2] [![0.6 docs][adb3]][ad3]

The `master` branch and the 0.6 release are rather old, it is recommended you use the `develop` branch.

[adb1]: https://img.shields.io/badge/docs-develop-blue.svg
[adb2]: https://img.shields.io/badge/docs-master-blue.svg
[adb3]: https://img.shields.io/badge/docs-0.6-blue.svg

[ad1]: https://www.amethyst.rs/doc/develop.html
[ad2]: https://www.amethyst.rs/doc/master.html
[ad3]: https://www.docs.rs/amethyst

## Usage

Please read the [online book][bk] for a comprehensive tutorial to using Amethyst.

[bk]: https://www.amethyst.rs/book/master/

## Getting started

To compile any of the examples run:
```
$ cargo run --example name_of_example
```
All available examples are listed under [examples][ex].

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
$ sudo yum install alsa-lib-devel
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

If you have a question, just ask on [Gitter][gt] or [Discord][di] and we'll help you and add it to the FAQ.

Other places you may want to check out are [r/rust_gamedev][rg] and [#rust-gamedev IRC][irc].

[faq]: https://github.com/amethyst/amethyst/wiki/Frequently-Asked-Questions
[gt]: https://gitter.im/amethyst/general
[di]: https://discord.gg/GnP5Whs
[rg]: https://www.reddit.com/r/rust_gamedev/
[irc]: https://botbot.me/mozilla/rust-gamedev/

## License

Amethyst is free and open source software distributed under the terms of both
the [MIT License][lm] and the [Apache License 2.0][la].

[lm]: docs/LICENSE-MIT
[la]: docs/LICENSE-APACHE

## Contributing

We are a community project that welcomes contribution from anyone. If you're
interested in helping out, please read the [CONTRIBUTING.md][cm] file before
getting started. Don't know what to hack on? Check our [active projects][pr], or search though [our issue tracker][it].

[cm]: docs/CONTRIBUTING.md
[pr]: https://github.com/amethyst/amethyst/projects
[it]: https://github.com/amethyst/amethyst/issues

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

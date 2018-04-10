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

**Warning: The book and tools repository are severely out of date at the moment. Amethyst is undergoing a lot of changes at the moment so if you are looking to use the library it would be better to just read the examples.**

This project is a *work in progress* and is very incomplete; pardon the dust!
Read a summary of what happened this past week at [*This Week in Amethyst*][tw].

[tw]: https://www.amethyst.rs/

### [Documentation][ar]

[ar]: https://www.amethyst.rs/doc/

## Vision

Amethyst is a fast, [data-oriented](https://www.amethyst.rs/book/master/book/glossary.html#data-oriented-programming), and data-driven game engine suitable
for rapid prototyping and iteration. It also tries to push the
[Rust programming language][rs] to its limits, driving further improvement and
hopefully attracting more game developers toward the young and vibrant Rust
ecosystem.

[rs]: https://www.rust-lang.org/

The engine's design draws much inspiration from the industrial-strength
[Bitsquid Engine][bs] (now called [Autodesk Stingray][sr]). However, Amethyst
does not aim to be API-compatible with it in any way. Some goals include:

[bs]: http://twvideo01.ubm-us.net/o1/vault/gdc2012/slides/Programming%20Track/Persson_Tobias_Flexible_Rendering.pdf.pdf
[sr]: http://stingrayengine.com/

* Simple game state management in the form of a [pushdown automaton][pa].
* Massively parallel architecture.
* [Correct entity-component-system model][em], with entities and prefabs largely
  defined in [Ron files][rn].
* Abstract scripting API that can be bound to a variety of embedded languages,
  such as [mruby][mr], [Lua][lu], etc.
* Renderer optimized for modern graphics APIs, e.g. Vulkan, Direct3D 12+, Metal.
* Easy integration with useful third-party game development libraries, e.g.
  [Piston][pi].
* Traditional "mega-editor" split into several
  [small but well-integrated tools][at], adhering to the [Unix philosophy][up].

[pa]: http://gameprogrammingpatterns.com/state.html#pushdown-automata
[em]: http://shaneenishry.com/blog/2014/12/27/misconceptions-of-component-based-entity-systems/
[rn]: https://github.com/ron-rs/ron
[mr]: http://mruby.org/
[lu]: http://www.lua.org/
[pi]: http://www.piston.rs/
[at]: https://github.com/amethyst/tools
[up]: https://en.wikipedia.org/wiki/Unix_philosophy

## Usage

Read the [online book][bk] for a comprehensive tutorial to using Amethyst. There
is also an online crate-level [API reference][ar].

[bk]: https://www.amethyst.rs/book/master/book/

## Getting started

To compile any of the examples run:
```
$ cargo run --example name_of_example
```
All available examples are listed under [examples][ex].

For a full-blown "Hello World" tutorial check out the [Getting Started][gs] chapter
in the book.

[ex]: examples/
[gs]: https://www.amethyst.rs/book/master/book/getting_started.html

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

If you have an easy question, just ask on [Gitter][gt] and we'll help you and add it to the FAQ.

Other places you may want to check out are [r/rust_gamedev][rg] and [#rust-gamedev IRC][irc].

[faq]: https://github.com/amethyst/amethyst/wiki/Frequently-Asked-Questions
[gt]: https://gitter.im/amethyst/general
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

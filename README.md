<img align="left" width="64px" src="book/images/amethyst_thumb.png" />

# Amethyst

[![Build Status][s1]][tc] [![Crates.io][s2]][ci] [![MIT License][s3]][ml] [![Join the chat][s4]][gc]

[s1]: https://travis-ci.org/amethyst/amethyst.svg?branch=master
[s2]: https://img.shields.io/badge/crates.io-0.2.1-orange.svg
[s3]: https://img.shields.io/badge/license-MIT-blue.svg
[s4]: https://badges.gitter.im/amethyst/general.svg

[tc]: https://travis-ci.org/amethyst/amethyst/
[ci]: https://crates.io/crates/amethyst/
[ml]: https://github.com/amethyst/amethyst/blob/master/COPYING
[gc]: https://gitter.im/orgs/amethyst/rooms

This project is a *work in progress* and is very incomplete; pardon the dust!
Read a summary of what happened this past week at [*This Week in Amethyst*][tw].

[tw]: https://thisweekinamethyst.wordpress.com/

## Vision

Amethyst aims to be a fast, data-oriented, and data-driven game engine suitable
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
* Massively parallel architecture, especially in rendering.
* [Correct entity-component-system model][em], with entities and prefabs largely
  defined in [YAML files][ya].
* Abstract scripting API that can be bound to a variety of embedded languages,
  such as [mruby][mr], [Lua][lu], etc.
* Renderer optimized for modern graphics APIs, e.g. Vulkan, Direct3D 12+, Metal.
* Easy integration with useful third-party game development libraries, e.g.
  [Piston][pi].
* Traditional "mega-editor" split into several
  [small but well-integrated tools][at], adhering to the [Unix philosophy][up].

[pa]: http://gameprogrammingpatterns.com/state.html#pushdown-automata
[em]: http://shaneenishry.com/blog/2014/12/27/misconceptions-of-component-based-entity-systems/
[ya]: http://www.yaml.org/
[mr]: http://mruby.org/
[lu]: http://www.lua.org/
[pi]: http://www.piston.rs/
[at]: https://github.com/amethyst/tools
[up]: https://en.wikipedia.org/wiki/Unix_philosophy

## Usage

Read the [online book][bk] for a comprehensive tutorial to using Amethyst. There
is also an online crate-level [API reference][ar].

[bk]: https://www.amethyst.rs/book/
[ar]: https://www.amethyst.rs/doc/amethyst/

## Quick Example

See the [Getting Started][gs] chapter in the book for the full-blown "Hello,
World!" tutorial. For the sake of brevity, you can generate an empty game
project with the [Amethyst CLI tool][ac] and build it. Follow along below:

[gs]: https://www.amethyst.rs/book/getting_started.html
[ac]: https://github.com/amethyst/tools/tree/master/src/cli

```
$ cargo install amethyst_tools
$ amethyst new mygame
$ cd mygame
$ amethyst run
```

If everything goes well, you should see the following print out to the terminal:

```
Game started!
Hello from Amethyst!
Game stopped!
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

## Contributing

We are a community project that welcomes contribution from anyone. If you're
interested in helping out, please read the [CONTRIBUTING.md][cm] file before
getting started. Don't know what to hack on? See the
[Development Roadmap][dr] on our wiki, or search though [our issue tracker][it].

[cm]: https://github.com/amethyst/amethyst/blob/master/CONTRIBUTING.md
[dr]: https://github.com/amethyst/amethyst/wiki/Roadmap
[it]: https://github.com/amethyst/amethyst/issues

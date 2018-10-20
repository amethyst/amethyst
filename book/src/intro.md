<div style="display:inline-block;width:100%">
    <img src="./images/amethyst_thumb.png" alt="Logo" width="96px" style="float:left;margin-right:15px"/>
    <h1>The Amethyst Engine</h1>
</div>

## Presentation

Howdy! This book will teach you everything you need to know about building video
games and interactive simulations with the Amethyst game engine. This engine is
written entirely in [Rust][rs], a safe and fast systems programming language,
and sports a clean and modern design. More correctly, though, Amethyst is
actually a collection of separate libraries and tools that collectively make up a
game engine.

[rs]: https://www.rust-lang.org/

Amethyst is free and open source software, distributed under a dual license of [MIT][ml]
and [Apache][al]. This means that the engine is given to you at no cost
and its source code is completely yours to tinker with. The code is available on
[GitHub][am]. Contributions and feature requests will always be welcomed!

[ml]: https://github.com/amethyst/amethyst/blob/master/docs/LICENSE-MIT
[al]: https://github.com/amethyst/amethyst/blob/master/docs/LICENSE-APACHE
[am]: https://github.com/amethyst/amethyst/tree/master

## Getting started

This book is split into several sections, with this introduction being the first. The others are:

* [Getting Started][gs] – Prepare your computer for Amethyst development.
* [Concepts][cc] – An overview of the concepts used in Amethyst. Recommended.
* [Pong Tutorial][pt] – Build a basic pong game in Rust.
* [Animation][anim] – Explains the architecture of the `amethyst_animation` crate.
* [Custom `GameData`][gad] – Shows you how to structure more complex games that need to change the system graph.
* [Glossary][gl] – Defines special terms used throughout the book.
* [Appendix A: Config Files][ax_a] – Shows you how to define your data in RON files.

[gs]: ./getting-started.html
[cc]: ./concepts/intro.html
[pt]: ./pong-tutorial.html
[anim]: ./animation.html
[gad]: ./game-data.html
[gl]: ./glossary.html
[ax_a]: ./appendices/a_config_files.html

Read the crate-level [API documentation][ad] for more details.

[ad]: https://www.amethyst.rs/doc/master/doc/amethyst/index.html

[db]: https://github.com/amethyst/amethyst/

## Motivation

Most of us have worked with quite a few game engines over the years, namely [Unity][un], [Unreal Engine][ud], [JMonkeyEngine][jme] and many more.
While they all are pretty solid solutions if you want to
build a quality game, each have their own pros and cons that you have to
weigh before using them, especially in regards to performance and scalability.

[un]: http://unity3d.com/
[ud]: https://www.unrealengine.com/
[jme]: http://jmonkeyengine.org/

We think that basing the Amethyst engine on good and modern principles will allow us to make an open source game engine that can actually be more performant than those engines.
Those principles are:

1. Modularity.

   Modularity is at the core of the [Unix philosophy][up], which proved itself to be an excellent way of developing software over the years.
   You will always be free to use the built-in modules, or to write your own and integrate them easily into the engine.
   Since modules are small and well integrated, it is easier to reason about what they do and how they relate to other modules.

2. Parallelism.

   Modern computers, even cheap ones, all have multithreading with multicore CPUs. We expect that over the years, there will be more and more opportunities for parallelism to improve performance.
   With a proper parallel engine, we are convinced that your game will be more and more performant over the years without even needing you to update it.

3. Data-oriented/Data-driven.

   Building your game around the data makes it really easy to prototype and quickly build a game.
   Complex behaviours like swapping assets during gameplay become a breeze, making testing and balancing a lot faster.

[up]: https://en.wikipedia.org/wiki/Unix_philosophy

## Contributing

We are always happy to welcome new contributors!

If you want to contribute, or have questions, let us know either on [GitHub][db], or on [Discord][di].

[di]: https://discord.gg/GnP5Whs

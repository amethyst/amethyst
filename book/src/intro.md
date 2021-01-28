<div class="splash">
   <img src="./images/logo.svg" class="splogo" alt="Logo" height="110px"/>
</div>
<div class="drop"></div>

Howdy! This book will teach you everything you need to know about building video
games and interactive simulations with the Amethyst game engine. This engine is
written entirely in [Rust][rs], a safe and fast systems programming language,
and sports a clean and modern design. More specifically, Amethyst is
an opinionated collection of separate libraries and tools that collectively make up a
game engine.

Amethyst is free and open source software, distributed under a dual license of [MIT][ml]
and [Apache][al]. This means that the engine is available to you at no cost
and its source code is yours to tinker with. The code is available on
[GitHub][am]. Contributions and feature requests will always be welcome!

## Getting started

This book is split in to several sections, with this introduction being the first. The others are:

- [Getting Started][gs] – Prepare your computer for Amethyst development.
- [Concepts][cc] – An overview of the concepts used in Amethyst. Recommended.
- [Pong Tutorial][pt] – Build a basic pong game in Rust.
- [Math] – A quick introduction to doing math with Amethyst.
- [Animation][anim] – Explains the architecture of the `amethyst_animation` crate.
- [Controlling `System` Execution][cse] – Shows you how to structure more complex games.
- [Glossary][gl] – Defines special terms used throughout the book.
- [Appendix A: Config Files][ax_a] – Shows you how to define your data in RON files.

Read the crate-level [API documentation][ad] for more details.

## Motivation

Most of us have worked with one game engine or another over the years, for example [Unity][un], [Unreal Engine][ud], and [JMonkeyEngine][jme].
While they are all solid solutions if you want to build a quality game, each have their own pros and cons that you have to
weigh before using them in regards to performance and scalability.

We believe that building the Amethyst engine on modern principles learned from our experiences with these engines will allow us to make an open source game engine that can actually be more performant than those mentioned above.
Those principles are:

1. ### Modularity

   Modularity is at the core of the [Unix philosophy][up], which has proven itself to be an excellent way of developing software over the years.
   You will always be free to use the built-in modules, or to write your own and integrate them in to the engine.
   Since modules are small and well integrated, it's easier to reason about what they do and how they relate to other modules.

1. ### Parallelism.

   Modern computers, even cheap ones, all have multithreading with multicore CPUs. There are now more opportunities for parallelism to improve performance than ever before.  With a proper parallel engine, your game will perform better on cheaper hardware, increasing your potential audience, as well as gaining performance benefits as technology improves.

1. ### Data-oriented/Data-driven.

   Building your game around the data makes it easy to prototype and build a game by reducing iteration cycle times.
   Complex processes like swapping assets during gameplay become a breeze, making testing, balancing, and refining a game a lot faster.

## Why use Amethyst?

While there are a lot of [great building blocks][awg] in the Rust ecosystem, using the Amethyst engine instead of building your own game engine has a lot of advantages.

First, the engine builds on the [Legion] library, which is the basis of the Parallel ECS architecture. For a great introduction to game development with Rust and an Entity Component System, see this [great talk by Catherine West](https://kyren.github.io/2018/09/14/rustconf-talk.html). Amethyst's take on ECS is described in the [concepts](./concepts/intro.md) section of this book.

Individual Amethyst sub-crates use other well-vetted libraries such as [Rodio] and [winit].

Amethyst's features have been glued together using those:

There are the obvious ones:

- Transformations
- Graphics
- Windowing
- Inputs
- Audio
- Etc...

And also the less known but also essential features:

- Animations
- Gltf
- Locales
- Networking

If you were not to use Amethyst, not only would you need to create all those features (or use pre-existing crates), but you would also need to glue the layers together.

Amethyst does all of this for you, so that you can focus on making your game instead of worrying about the low-level details.

Futhermore, because of the architecture of Amethyst, almost all the parts are both configurable and replaceable. This means that if you do want to change something to suit your needs, there's always a way to do it.

For example, the [rodio](https://github.com/tomaka/rodio) crate is currently used for the audio features in the engine, but if you would rather use something more complex or a custom solution, all you have to do is add some glue that moves the data coming from Specs into the library that you are using to play and control the audio, without even having to touch the engine code!

## Contributing

We are always happy to welcome new contributors!

To know where to start, we suggest you read our [contribution guidelines](https://github.com/amethyst/amethyst/blob/master/docs/CONTRIBUTING.md)

If you want to contribute, or have questions, let us know either on [GitHub][db], or on [Discord][di].

[ad]: https://docs.amethyst.rs/stable/amethyst/index.html
[al]: https://github.com/amethyst/amethyst/blob/master/docs/LICENSE-APACHE
[am]: https://github.com/amethyst/amethyst
[anim]: ./animation.html
[awg]: http://arewegameyet.com/
[ax_a]: ./appendices/a_config_files.html
[cc]: ./concepts/intro.html
[cse]: ./controlling_system_execution.html
[db]: https://github.com/amethyst/amethyst/
[di]: https://discord.gg/amethyst
[gl]: ./glossary.html
[gs]: ./getting-started.html
[jme]: http://jmonkeyengine.org/
[legion]: https://github.com/amethyst/legion
[math]: ./math.html
[ml]: https://github.com/amethyst/amethyst/blob/master/docs/LICENSE-MIT
[pt]: ./pong-tutorial.html
[rodio]: https://github.com/RustAudio/rodio
[rs]: https://www.rust-lang.org/
[ud]: https://www.unrealengine.com/
[un]: http://unity3d.com/
[up]: https://en.wikipedia.org/wiki/Unix_philosophy
[winit]: https://github.com/rust-windowing/winit

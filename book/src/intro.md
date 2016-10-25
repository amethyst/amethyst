<img src="./images/amethyst_thumb.png" alt="Logo" width="96px" style="float:left;margin-right:15px"/>

# The Amethyst Engine

> Note: This project is a *work in progress* and is very incomplete. Pardon the
> dust!

Howdy! This book will teach you everything you need to know about building video
games and interactive simulations with the Amethyst game engine. This engine is
written entirely in [Rust][rs], a safe and fast systems programming language,
and sports a clean and modern design. More correctly, though, Amethyst is
actually a suite of separate libraries and tools that collectively make up a
game engine.

[rs]: https://www.rust-lang.org/

Amethyst is free and open source software, distributed under the
[MIT License][ml]. This means that the engine is given to you at no cost and its
source code is completely yours to tinker with. The code is available on
[GitHub][am]. Contributions and feature requests are welcome!

[ml]: https://github.com/amethyst/amethyst/blob/master/COPYING
[am]: https://github.com/amethyst/amethyst

This book is split into seven sections. This page is the first. The others are:

* [Getting Started][gs] – Prepare your computer for Amethyst development.
* [A Simple Application][sa] – Build a basic pong game in Rust.
* Effective Amethyst – Learn how to write complex games and applications.
* Scripting – Jump into scripted development with [Ruby][rb].
* [Glossary][gl] - Defines special terms used throughout the book.
* Bibliography – Read the works that influenced the design.

[gs]: ./getting_started.html
[sa]: ./simple_application.html
[rb]: https://www.ruby-lang.org/
[gl]: ./glossary.html

Read the crate-level [API documentation][ad] for more details.

[ad]: https://www.amethyst.rs/doc/master/amethyst/

## Why are you building this?

I've worked with a few game engines over the years, namely [Unity][un] and the
[Unreal Development Kit][ud], and both are pretty solid solutions if you want to
build a quality game. But each have their own pros and cons that you have to
weigh before using them, especially in regards to performance and scalability.

[un]: http://unity3d.com/
[ud]: https://www.unrealengine.com/

One engine I've always admired as a programmer but never had a chance to play
with is the [Bitsquid Engine][bs] (now called [Autodesk Stingray][as]). It's
fast, forward-thinking, highly parallel, and data-driven. It seems like a
wonderful platform for rapid prototyping. I've wanted to play around with a
Bitsquid-like engine for a while, but I couldn't find any open-source
equivalents out there. Most of those I did find stuck to outdated design
patterns and lacked the multi-core scalability I was looking for. So I set out
to write my own.

[bs]: http://twvideo01.ubm-us.net/o1/vault/gdc2012/slides/Programming%20Track/Persson_Tobias_Flexible_Rendering.pdf.pdf
[as]: http://stingrayengine.com/
[bl]: http://bitsquid.blogspot.com/

In short, I am writing Amethyst to scratch three of my own itches:

1. Teach myself Rust, game development, and computer graphics in their purest
   form, rather than through the lens of a particular game engine.
2. Write a modular, parallel, data-oriented, and data-driven engine suited for
   rapid prototyping that demands (a little) less boilerplate from the user.
3. Build a toolset that splits up the traditional "mega-editor" into several
   [small but well-integrated tools][at], adhering to the [Unix philosophy][up].

[at]: https://github.com/ebkalderon/amethyst_tools
[up]: https://en.wikipedia.org/wiki/Unix_philosophy

## Contributing

The Markdown source files from which this book is generated can be found
[on GitHub][md]. Pull requests are welcome!

[md]: https://github.com/amethyst/amethyst/tree/master/book/src

<img align="left" src="./images/amethyst_thumb.png" />

The Amethyst Engine

> Note: This project is a *work in progress* and is very incomplete. Pardon the
> dust!

Howdy! This book will teach you everything you need to know about building video
games and interactive simulations with the Amethyst game engine. This engine is
written entirely in [Rust][rs], a safe and fast systems programming language,
and sports a clean and modern design. More correctly, though, Amethyst is
actually a suite of separate libraries and tools that collectively make up a
game engine.

[rs]: https://www.rust-lang.org/

Amethyst is free and open source software, licensed under the [MIT License][ml].
This means that the engine is given to you at no cost and its source code is
completely yours to tinker with. The code is available on [GitHub][am].
Contributions and feature requests are welcome!

[ml]: https://github.com/ebkalderon/amethyst/blob/master/COPYING
[am]: https://github.com/ebkalderon/amethyst

This book is split into seven sections. This page is the first. The others are:

* [Getting Started][gs] – Prepare your computer for Amethyst development.
* [A Simple Application][sa] – Build a basic pong game in Rust.
* Effective Amethyst – Learn how to write complex games and applications.
* Scripting – Jump into script development with [Ruby][rb].
* Glossary - Defines special terms used throughout the book.
* Bibliography – Read the works that influenced the design.

[gs]: ./getting_started.html
[sa]: ./simple_application.html
[rb]: https://www.ruby-lang.org/

Read the crate-level [API documentation][ad] for more details.

[ad]: http://ebkalderon.github.io/amethyst/doc/amethyst/

## Why are you building this?

The video game industry is getting bigger every year, and it has actually been
[outpacing Hollywood for years][hw]. As game studios grow, their toolset grows
to match. Though we now have access to some great mostly free game development
tools, their workflows are [clunky][ue] and unfriendly to throwaway
experimentation and iteration. Most of these tools are also pretty opaque to how
they work internally, and those that are open don't usually adhere to modern
design patterns well (think pre-C++11 idioms and convoluted class hierarchies).

[hw]: https://www.quora.com/Who-makes-more-money-Hollywood-or-the-video-game-industry
[ue]: http://cdn.dbolical.com/videos/engines/1/1/456/Unreal_Engine_4_Features_Trailer_--_GDC_2014.mp4.jpg

In short, I wrote Amethyst to scratch three of my own itches:

1. Teach myself game development and computer graphics in its purest form,
   rather than through the lens of a particular game engine.
2. Write a fast, modular, data-oriented and data-driven engine suited for rapid
   prototyping that demands (a little) less boilerplate from the user.
3. Build a toolset that splits up the traditional "mega-editor" into several
   small but well-integrated tools, adhering to the [Unix philosophy][up].

[up]: https://en.wikipedia.org/wiki/Unix_philosophy

## Contributing

The Markdown source files from which this book is generated can be found
[on GitHub][md]. Pull requests are welcome!

[md]: https://github.com/ebkalderon/amethyst/tree/master/book/src


# Amethyst

Howdy! This book will teach you everything you need to know about building video games and interactive simulations with the Amethyst software development kit. Amethyst is a clean and modern game engine written in [Rust][rs], a safe and fast systems programming language. But more correctly, Amethyst is a whole suite of libraries and tools that collectively make up an engine.

[rs]: https://www.rust-lang.org/

This book is split into seven sections. This page is the first. The others are:

* [Getting Started][gs] – Prepare your computer for Amethyst development.
* Writing A Simple Application – Build a basic pong game in Rust.
* Effective Amethyst – Learn how to write complex games and applications.
* Scripting – Jump into cross-platform development with [Ruby][rb].
* [Internals][in] – Peek into Amethyst's innards and learn how it works.
* Bibliography – Read the works that influenced the design.

[gs]: ./getting_started.html
[rb]: https://www.ruby-lang.org/
[in]: ./internals.html

Read the [API documentation][ad].

[ad]: https://github.com/ebkalderon/amethyst/doc/index.html

## Why the hell did you build this?

The video game industry is getting bigger every year, and it has actually been [outpacing Hollywood for years][hw]. As game studios grow, their toolset grows to match. Though we have got some incredible game development tools in our hands (and often for free, I might add), the process has become increasingly [clunky][ue] and unfriendly to throwaway experimentation and iteration. The tools available are also pretty opaque to how they work internally, and if they are open, they don't usually adhere to modern design patterns well (think pre-C++11 idioms and convoluted class hierarchies).

[hw]: https://www.quora.com/Who-makes-more-money-Hollywood-or-the-video-game-industry
[ue]: http://cdn.dbolical.com/videos/engines/1/1/456/Unreal_Engine_4_Features_Trailer_--_GDC_2014.mp4.jpg

In short, I wrote Amethyst to scratch three of my own itches:

1. Teach myself game development and computer graphics in its purest form,
   rather than through the lens of a particular game engine.
2. Write a fast, modular, data-driven engine suited for rapid prototyping that
   demands (a little) less boilerplate.
3. Build a toolset that splits up the traditional "mega-editor" into several
   small but well-integrated tools, adhering to the [Unix philosophy][up].

[up]: https://en.wikipedia.org/wiki/Unix_philosophy

## Contributing

The Markdown source files from which this book is generated can be found [on GitHub][md]. Pull requests are welcome!

[md]: https://github.com/ebkalderon/amethyst/tree/master/book/src


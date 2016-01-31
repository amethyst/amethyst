# Getting Started

These instructions will help you set up a working Amethyst game development
environment. Then we'll test this environment by compiling a simple "Hello,
World" application.

Amethyst should compile hassle-free on any platform properly supported by the
Rust compiler. Here are the system requirements (they're pretty modest):

* Minimum:
  * CPU: 1GHz, the more cores the better
  * RAM: 512 MiB
  * HDD: 17 MiB free disk space
  * OS: Windows Vista and newer, Linux, BSD, Mac OS X
  * Rust: Nightly (1.6.0 or newer)
* Renderer Backends:
  * OpenGL 4.0 or newer

## Setting Up

> Note: This guide assumes you have nightly Rust and Cargo installed, and also
> have a working Internet connection. Please take care of these prerequisites
> first before proceeding.

There are two ways to get started working with Amethyst:

1. [Use the Amethyst CLI tool to generate a new game project][as].
2. [Create the Cargo crate and the "resources" folder structure yourself][ms].

[as]: ./getting_started/automatic_setup.html
[ms]: ./getting_started/manual_cargo_setup.html

Since we're just getting started, it's fastest and highly recommended to use
[Amethyst CLI][ac], which is included in the [amethyst_tools][at] crate. If
you're of the intrepid type, you may go the vanilla Cargo route if you wish.

[ac]: https://github.com/ebkalderon/amethyst_tools/tree/master/src/cli
[at]: https://github.com/ebkalderon/amethyst_tools

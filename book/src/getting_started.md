# Getting Started

These instructions will help you set up a working Amethyst development
environment. Then we'll test this environment by compiling a simple "Hello,
World" application.

Amethyst should compile hassle-free on any platform properly supported by the
Rust compiler. Here are the system requirements:

* Minimum:
  * CPU: 1GHz, the more cores the better
  * RAM: 512 MiB
  * HDD: 30 MiB free disk space
  * OS: Windows Vista and newer, Linux, BSD, Mac OS X
  * Rust: Nightly (1.6.0 or newer)
* Renderer Backends:
  * OpenGL 4.0 or newer

## Setting Up

> Note: This guide assumes you have nightly Rust and Cargo installed, and also a
> working Internet connection. Please take care of these first before
> proceeding.

There are two ways to get started working with Amethyst:

1. [Use the `amethyst_cli` tool to generate a new Amethyst project][ac].
2. [Create the Cargo crate and the "resources" folder structure yourself][mc].

[ac]: ./getting_started/automatic_setup.html
[mc]: ./getting_started/manual_cargo_setup.html

Since we're just getting started, it's fastest and highly recommended to use
`amethyst_cli`. If you're of the intrepid type, you may go the vanilla Cargo
route if you wish.

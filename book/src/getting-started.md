# Getting started

## Setting up Rust

We recommend using [rustup][ru] to easily install the latest stable version of rust.
Instructions should be on screen once rustup is downloaded.

[ru]: https://rustup.rs

> **Updating Rust:** If you already have Rust installed, make sure you're using the
  latest version by running `rustup update`.

We recommend using the stable version of Rust, as Rust nightlies tend to break rather
often.

> **Using the stable toolchain:** Rustup can be configured to default to the stable
  toolchain by running `rustup default stable`.

## Required dependencies

Please check the dependencies section of the
[README.md](https://github.com/amethyst/amethyst/blob/master/README.md#dependencies)
for details on what dependencies are required for compiling Amethyst.

Please note that you need to have a functional graphics driver installed.
If you get a panic about the renderer unable to create the rendering context
when trying to run an example, a faulty driver installation could be the issue.

## Setting up Amethyst

You can either use the [Amethyst CLI][cl] or `cargo` to set up your project.

### Amethyst CLI (Easiest)

If you wish to use the Amethyst cli tool, you can install it like so

```norun
cargo install amethyst_tools
```

and then run

```norun
amethyst new <game-name>
```

you should get `Cargo.toml`, `src/main.rs` and `config/display.ron`.

### Starter Project

If you want to get running as quickly as possibly and start playing around with Amethyst, you can also use a starter project. These are specifically made for certain types of games, and will set you up with the groundwork needed to start right away.  
The `README.md` file on these will include everything you need to know to run the starter project.

> **Note:** Right now, the only starter available is for 2D games. This will expand over time, and offer more options for different types of games.

* [2D Starter](https://github.com/amethyst/amethyst-starter-2d)

### Cargo (Manual)

In case you're doing this with `cargo`, here's what you need to do:

* Add `amethyst` as dependency in your `Cargo.toml`.
* Create a `config` folder and put a `display.ron` in it.
* (Optional) Copy the code from one of amethyst's examples.

### Important note on versioning

Amethyst is divided in two major versions:

* The released crates.io version, which is the latest version available on crates.io
* The git (master) version, which is the current unreleased development snapshot of Amethyst available on [Github][agit]

> **Note:** You can see which version you're currently looking at by checking the URL
  in your browser. The book / documentation for `master` contains "master" in the address,
  the crates.io version is called "stable".

Depending on the book version that you choose to read, make sure that the amethyst version in your Cargo.toml matches that.

For the released crates.io version, you should have something like this:

```rust,ignore
[dependencies]
amethyst = "LATEST_CRATES.IO_VERSION"
```
The latest crates.io version can be found [here](https://crates.io/crates/amethyst).

If you want to use the latest unreleased changes, your Cargo.toml file should look like this:

```rust,ignore
[dependencies]
amethyst = { git = "https://github.com/amethyst/amethyst", rev = "COMMIT_HASH" }
```

The commit hash part is optional. It indicates which specific commit your project uses, to prevent unexpected breakage when we make changes to the git version.

[agit]: https://github.com/amethyst/amethyst
[cl]: https://github.com/amethyst/tools


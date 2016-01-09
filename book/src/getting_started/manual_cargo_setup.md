# Manual Setup With Cargo

For those of you who prefer working with Cargo directly, like any other Rust
project, you can. However, since this is just a "Hello, World" program, I would
strongly recommend using the [`amethyst_cli` tool][ac] instead. It will save you
a whole lot of time.

[ac]: ./getting_started/automatic_setup.html

### From Crates.io

Create a new Rust executable project with `cargo new --bin hello_world` and add
the following lines to your "Cargo.toml":

```toml
[dependencies]
amethyst = "*"
```

### From Git

If you don't want to get Amethyst from Crates.io, you can download and compile
Amethyst from the Git repository:

```
$ git clone https://github.com/ebkalderon/amethyst.git
$ cd amethyst
$ cargo build
```

Then, in your crate's "Cargo.toml", specify the location of the `amethyst`
library yourself:

```toml
[dependencies.amethyst]
path = "../path/to/amethyst/folder/"
```

## Resources Folder

Every Amethyst game project must have a top-level folder called "resources".
This is where your game assets are stored. In your project's root folder, create
the following folder structure:

* **resources**/
  * **entities**/
  * **prefabs**/
  * config.yml
  * input.yml

And in the "config.yml" file, copy and paste this YAML configuration:

```
---
logging:
    file_path: "hello_world.log"
    output_verbosity: medium
    logging_verbosity: debug 

display:
    brightness: 1.0
    fullscreen: false
    resolution: [1024, 768]
```

The "input.yml" file, which normally holds key binding data, can be left blank
since this is a simple application with no user interaction.

## All Set!

Whew, we're done! Let's move on and write our first
["Hello, World" program][hw].

[hw]: ./getting_started/hello_world.html

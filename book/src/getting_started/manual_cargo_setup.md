# Manual Setup With Cargo

For those of you who prefer working with Cargo directly, like any other Rust
project, you can. However, since this is just a "Hello, World" program, I would
strongly recommend using the [Amethyst CLI tool or cargo with the project template][as] instead. It will save you
a whole lot of time.

[as]: ./getting_started/automatic_setup.html

### From Crates.io

Create a new Rust executable project with `cargo new --bin hello_world` and add
the following lines to your "Cargo.toml":

```toml
[dependencies]
amethyst = "*"
```

### From Git via cargo

If you don't want to get Amethyst from Crates.io but directly from the
Git repository, you can add the following lines to your "Cargo.toml":

```toml
[dependencies]
amethyst = { git = "https://github.com/amethyst/amethyst.git" }
```

See the [crates guide][crg] for more information on the Cargo manifest file.

[crg]: http://doc.crates.io/specifying-dependencies.html#specifying-dependencies-from-git-repositories

### From Git via clone

Instead of using the above, you can also clone the entire SDK from the
Git repository. Once you're done, create a new Cargo project and `cd` into it.

```
$ git clone https://github.com/amethyst/amethyst.git
$ cargo new --bin hello_world
$ cd hello_world
```

In your "Cargo.toml" manifest, add the local `amethyst` crate and its sub-crates
as dependencies:

```toml
[dependencies.amethyst]
path = "../path/to/amethyst/"
```

## Resources Folder

Every Amethyst game project must have a top-level folder called "assets".
This is where your game assets are stored. In your project's root folder, create
the following folder structure:

* **assets**/
  * **meshes**/
  * **textures**/
  * **entities**/
  * **prefabs**/
  * config.yml
  * input.yml

And in the "config.yml" file, copy and paste this YAML configuration:

```yaml
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

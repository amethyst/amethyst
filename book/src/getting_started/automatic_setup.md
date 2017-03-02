# Automatic Setup Using The CLI Tool

An easy way to set up Amethyst and manage your project is with the
[amethyst_tools][at] crate. If you want to set up a game project in Cargo by
hand, follow along with [the next section][ci] instead.

[at]: https://github.com/amethyst/tools
[ci]: ./getting_started/manual_cargo_setup.html

To install the toolchain and generate a new project with the CLI tool, follow
along in your terminal:

```bash
$ cargo install amethyst_tools
$ amethyst new hello_world
```

That's it! You should now have a customized Cargo project set up in a directory
called "hello_world". `cd` into it, and you should find the following file
structure:

* **hello_world**/
  * **assets**/
    * Lots of junk...
  * **src**/
    * main.rs
  * Cargo.toml

If you do have nightly cargo installed, you can also use

```bash
$ cargo new mygame --template https://github.com/amethyst/project_template
```

to set up a project.

You're all set! Skip forward to [section 2.3][hw] to see your setup in action.

[hw]: ./getting_started/hello_world.html

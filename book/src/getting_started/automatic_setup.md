# Automatic Setup Using The CLI Tool

An easy way to set up Amethyst and manage your project is with the
[amethyst_cli][ac] crate. If you want to set up an Amethyst game project in Cargo by
hand, follow along with [the next section][ci] instead.

[ac]: https://github.com/ebkalderon/amethyst_cli
[ci]: ./getting_started/manual_cargo_setup.html

To install the `amethyst_cli` tool, follow along in your terminal:

```
cargo install amethyst_cli
amethyst new hello_world
```

That's it! You should now have a customized Cargo project set up in a directory
called "hello_world". `cd` into it, and you should find the following file
structure:

* **hello_world**/
  * **resources**/
    * Lots of junk...
  * **src**/
    * main.rs
  * Cargo.toml

You're all set! Skip forward to [section 2.3][hw] to see your setup in action.

[hw]: ./getting_started/hello_world.html

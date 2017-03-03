# Automatic Setup Using Cargo

This section describes how to use the project template to create a project. If you want to create your project by hand, follow the [manual setup][ci] instead.

[ci]: ./getting_started/manual_setup.html

To setup your project, just enter these commands:

```bash
$ cargo +nightly new mygame --template https://github.com/amethyst/project_template
```

**Note:** This assumes you are using [rusutp.rs](https://github.com/rust-lang-nursery/rustup.rs). If you are using the plain nightly, leave out the `+nightly`.

That's it! You should now have a customized Cargo project set up in a directory
called "hello_world". `cd` into it, and you should find the following file
structure:

* **hello_world**/
  * **assets**/
    * Lots of junk...
  * **src**/
    * main.rs
  * Cargo.toml

You're all set! Skip forward to [section 2.3][hw] to see your setup in action.

[hw]: ./getting_started/hello_world.html

# Getting started

You can either use the [Amethyst CLI][cl] or just cargo to set up your project.
After executing

```
amethyst new game
```

you should get `Cargo.toml`, `src/main.rs` and `resources/display_config.ron`.
In case you're doing this with `cargo`, here's what you need to do:

* Add `amethyst` as dependency in your `Cargo.toml`.
* Create a `resources` folder and put a `display_config.ron` in it.
* Start with one of the [examples][ex] from the Amethyst repository (e.g. `window`)
  for the source code. Watch out to use the right example for the version of Amethyst
  you specified in `Cargo.toml`.

We don't have any tutorials yet, but there's a [Gitter room][gi] where you can
ask in case you want an explanation for something. If you'd like to help out,
a tutorial would be much appreciated!

[cl]: https://github.com/amethyst/tools
[gi]: https://gitter.im/amethyst/general

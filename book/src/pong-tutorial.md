# Pong Tutorial

To get a better feeling for how Amethyst works, we're going to implement a 
Pong clone. You can find a [full Pong example][pong] (our end goal) in Amethyst's 
examples folder. This tutorial breaks that project up into discrete steps so 
it's easier to understand what everything is doing. If you've cloned the 
Amethyst repo, you can run any of the examples like so:

```norun
cargo run --example pong_tutorial_01
```

The main difference between real game code and the example code is where the 
`resources` and `assets` folders are located.

For instance, in the pong_tutorial_01 example we have:

```rust,norun
use amethyst::utils::application_root_dir;

let app_root = application_root_dir();

let path = format!(
    "{}/examples/pong_tutorial_01/resources/display_config.ron",
    app_root
);
```

But for your own project you'll probably want something like this:

```rust,norun
let path = "./resources/display_config.ron";
```

The reason `application_root_dir` is used in our examples is that the
required resources are not in the working directory, but rather somewhere
in the `examples` folder. However, in an actual game it would be bad to
hardcode the path of the resources to the directory that was used to compile
the application, because then the game cannot be moved (or else it wouldn't
find the resources). That's why you should stick to a relative path for your
game, as shown above.

[pong]: https://github.com/amethyst/amethyst/tree/master/examples/pong


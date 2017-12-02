# Pong Tutorial

To get a better feeling for how Amethyst works, we're going to implement a 
Pong clone. You can find a full Pong example (our end goal) in Amethyst's 
examples folder. This tutorial breaks that project up into discrete steps so 
it's easier to understand what everything is doing. You can run any of the 
examples like so:

```cargo run --example pong_tutorial_01```

The main difference between your code and the example code is where the 
`resources` and `assets` folders are located.

For instance, in the pong_tutorial_01 example we have:

```rust,ignore
let path = format!(
    "{}/examples/pong_tutorial_01/resources/display_config.ron",
    env!("CARGO_MANIFEST_DIR"));
```

But for your own project you'll want something like this:

```rust,ignore
let path = format!(
    "{}/resources/display_config.ron",
    env!("CARGO_MANIFEST_DIR")
);
```

## Chapters

* [Opening (and closing!) a window][01]
* [Drawing the paddles][02]

[01]: ./pong_tutorial/pong_tutorial_01.html
[02]: ./pong_tutorial/pong_tutorial_02.html


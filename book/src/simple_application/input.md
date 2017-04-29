# Processing user input

In this stage we are just going to get input information into our system. The
input information will come to us as a stream of events, but it will be easier
for us to store key state so we can access it from our systems. To do this, we
have a key-value data structure where there is an entry for each key.

Firstly, we need to register the `InputHandler` type in our `onStart` method.

```rust
world.add_resource::<InputHandler>(InputHandler::new());
```

This registers the input handler as a `Resource`, this is a piece of data that
is shared between all entities. TODO I need to check this is correct.

The other step is to pass input events to our `InputHandler` instance. To do
this we implement the `handle_events` trait function on our `Pong` state.

```rust
fn handle_events(&mut self,
                 events: &[WindowEvent],
                 world: &mut World,
                 _: &mut AssetManager,
                 _: &mut Pipeline)
                 -> Trans {
    use amethyst::ecs::Gate;
    use amethyst::ecs::resources::InputHandler;

    // Grab the input handler and run any input events through it.
    let input = world.write_resource::<InputHandler>();
    input.pass().update(events);

    // Quit on Esc or window close request
    for e in events {
        match **e {
            Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape))
                | Event::Closed => return Trans::Quit,
            _ => (),
        }
    }
    Trans::None
}
```

We're now loading input state into the `InputHandler`! We'll see how to use it
in a later stage.

> If you run the app now, you'll be able to quit by either pressing `Esc`, or
> clicking on the window close button, if your windowing system provides these.

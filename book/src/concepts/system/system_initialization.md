# System Initialization

Systems may need to access resources from the `World` in order to be
instantiated. For example, obtaining a `ReaderId` to an `EventChannel` that
exists in the `World`. When there is an existing event channel in the `World`, a
`System` should register itself as a reader of that channel instead of replacing
it, as that invalidates all other readers.

In Amethyst, the `World` that the application begins with is populated with a
number of default resources -- event channels, a thread pool, a frame limiter,
and so on.

Given the default resources begin with special limits, we need a way to pass the
`System` initialization logic through to the application, including parameters to
the `System`'s constructor. This is information the `SystemDesc` trait captures.

For each `System`, an implementation of the `SystemDesc` trait specifies the
logic to instantiate the `System`. For `System`s that do not require special
initialization logic, the `SystemDesc` derive automatically implements the
`SystemDesc` trait on the system type itself:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::{
    core::SystemDesc,
    derive::SystemDesc,
    ecs::{System, SystemData, World},
};

#[derive(SystemDesc)]
struct SystemName;

impl<'a> System<'a> for SystemName {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {
        println!("Hello!");
    }
}
```

The [`SystemDesc` derive] page demonstrates the use cases supported by the
`SystemDesc` derive. For more complex cases, the
[Implementing the `SystemDesc` Trait] page explains how to implement the
`SystemDesc` trait.

[`SystemDesc` derive]: ./system_desc_derive.html
[Implementing the `SystemDesc` Trait]: ./implementing_the_system_desc_trait.html

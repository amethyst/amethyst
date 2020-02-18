# `SystemDesc` Derive

The `SystemDesc` derive supports the following cases when generating a `SystemDesc` trait implementation:

* Parameters to pass to the system constructor.
* Fields to skip -- defaulted by the system constructor.
* Registering a `ReaderId` for an `EventChannel<_>` in the `World`.
* Registering a `ReaderId` to a component's `FlaggedStorage`.
* Inserting a resource into the `World`.

If your system initialization use case is not covered, please see the
[Implementing the `SystemDesc` Trait] page.

## Passing parameters to system constructor

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{System, SystemData},
# };
#
#[derive(SystemDesc)]
#[system_desc(name(SystemNameDesc))]
pub struct SystemName {
    field_0: u32,
    field_1: String,
}

impl SystemName {
    fn new(field_0: u32, field_1: String) -> Self {
        SystemName { field_0, field_1 }
    }
}
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
```

<details>
<summary>Generated code</summary>

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{System, SystemData},
# };
#
# pub struct SystemName {
#     field_0: u32,
#     field_1: String,
# }
#
# impl SystemName {
#     fn new(field_0: u32, field_1: String) -> Self {
#         SystemName { field_0, field_1 }
#     }
# }
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
#
/// Builds a `SystemName`.
#[derive(Default, Debug)]
pub struct SystemNameDesc {
    field_0: u32,
    field_1: String,
}

impl SystemNameDesc {
    fn new(field_0: u32, field_1: String) -> Self {
        SystemNameDesc { field_0, field_1 }
    }
}

impl<'a, 'b> ::amethyst::core::SystemDesc<'a, 'b, SystemName> for SystemNameDesc {
    fn build(self, world: &mut ::amethyst::ecs::World) -> SystemName {
        <SystemName as ::amethyst::ecs::System<'_>>::SystemData::setup(world);

        SystemName::new(self.field_0, self.field_1)
    }
}
```

</details>

## Fields to skip -- defaulted by the system constructor

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{System, SystemData},
# };
#
#[derive(SystemDesc)]
#[system_desc(name(SystemNameDesc))]
pub struct SystemName {
    #[system_desc(skip)]
    field_0: u32,
    field_1: String,
}

impl SystemName {
    fn new(field_1: String) -> Self {
        SystemName { field_0: 123, field_1 }
    }
}
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
```

<details>
<summary>Generated code</summary>

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{System, SystemData},
# };
#
# pub struct SystemName {
#     field_0: u32,
#     field_1: String,
# }
#
# impl SystemName {
#     fn new(field_1: String) -> Self {
#         SystemName { field_0: 123, field_1 }
#     }
# }
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
#
/// Builds a `SystemName`.
#[derive(Default, Debug)]
pub struct SystemNameDesc {
    field_1: String,
}

impl SystemNameDesc {
    fn new(field_1: String) -> Self {
        SystemNameDesc { field_1 }
    }
}

impl<'a, 'b> ::amethyst::core::SystemDesc<'a, 'b, SystemName> for SystemNameDesc {
    fn build(self, world: &mut ::amethyst::ecs::World) -> SystemName {
        <SystemName as ::amethyst::ecs::System<'_>>::SystemData::setup(world);

        SystemName::new(self.field_1)
    }
}
```

</details>

**Note:** If there are no field parameters, the `SystemDesc` implementation
will call  `SystemName::default()`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{System, SystemData},
# };
#
#[derive(Default, SystemDesc)]
#[system_desc(name(SystemNameDesc))]
pub struct SystemName {
    #[system_desc(skip)]
    field_0: u32,
}
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
```

<details>
<summary>Generated code</summary>

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{System, SystemData},
# };
#
# #[derive(Default)]
# pub struct SystemName {
#     field_0: u32,
# }
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
#
/// Builds a `SystemName`.
#[derive(Debug)]
pub struct SystemNameDesc {}

impl Default for SystemNameDesc {
    fn default() -> Self {
        SystemNameDesc {}
    }
}

impl<'a, 'b> ::amethyst::core::SystemDesc<'a, 'b, SystemName> for SystemNameDesc {
    fn build(self, world: &mut ::amethyst::ecs::World) -> SystemName {
        <SystemName as ::amethyst::ecs::System<'_>>::SystemData::setup(world);

        SystemName::default()
    }
}
```

</details>

## Registering a `ReaderId` for an `EventChannel<_>` in the `World`

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{System, SystemData},
#     shrev::{EventChannel, ReaderId},
#     ui::UiEvent,
# };
#
#[derive(SystemDesc)]
#[system_desc(name(SystemNameDesc))]
pub struct SystemName {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<UiEvent>,
}

impl SystemName {
    fn new(reader_id: ReaderId<UiEvent>) -> Self {
        SystemName { reader_id }
    }
}
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
```

<details>
<summary>Generated code</summary>

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{System, SystemData},
#     shrev::{EventChannel, ReaderId},
#     ui::UiEvent,
# };
#
# pub struct SystemName {
#     reader_id: ReaderId<UiEvent>,
# }
#
# impl SystemName {
#     fn new(reader_id: ReaderId<UiEvent>) -> Self {
#         SystemName { reader_id }
#     }
# }
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
#
/// Builds a `SystemName`.
#[derive(Debug)]
pub struct SystemNameDesc;

impl Default for SystemNameDesc {
    fn default() -> Self {
        SystemNameDesc {}
    }
}

impl<'a, 'b> ::amethyst::core::SystemDesc<'a, 'b, SystemName> for SystemNameDesc {
    fn build(self, world: &mut ::amethyst::ecs::World) -> SystemName {
        <SystemName as ::amethyst::ecs::System<'_>>::SystemData::setup(world);

        let reader_id = world
            .fetch_mut::<EventChannel<UiEvent>>()
            .register_reader();

        SystemName::new(reader_id)
    }
}
```

</details>

## Registering a `ReaderId` to a component's `FlaggedStorage`

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{storage::ComponentEvent, System, SystemData, WriteStorage},
#     shrev::{EventChannel, ReaderId},
#     ui::UiResize,
# };
#
#[derive(SystemDesc)]
#[system_desc(name(SystemNameDesc))]
pub struct SystemName {
    #[system_desc(flagged_storage_reader(UiResize))]
    resize_events_id: ReaderId<ComponentEvent>,
}

impl SystemName {
    fn new(resize_events_id: ReaderId<ComponentEvent>) -> Self {
        SystemName { resize_events_id }
    }
}
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
```

<details>
<summary>Generated code</summary>

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{storage::ComponentEvent, System, SystemData, WriteStorage},
#     shrev::{EventChannel, ReaderId},
#     ui::UiResize,
# };
#
# pub struct SystemName {
#     resize_events_id: ReaderId<ComponentEvent>,
# }
#
# impl SystemName {
#     fn new(resize_events_id: ReaderId<ComponentEvent>) -> Self {
#         SystemName { resize_events_id }
#     }
# }
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ();
#     fn run(&mut self, data: Self::SystemData) {}
# }
#
/// Builds a `SystemName`.
#[derive(Debug)]
pub struct SystemNameDesc;

impl Default for SystemNameDesc {
    fn default() -> Self {
        SystemNameDesc {}
    }
}

impl<'a, 'b> ::amethyst::core::SystemDesc<'a, 'b, SystemName> for SystemNameDesc {
    fn build(self, world: &mut ::amethyst::ecs::World) -> SystemName {
        <SystemName as ::amethyst::ecs::System<'_>>::SystemData::setup(world);

        let resize_events_id = WriteStorage::<UiResize>::fetch(&world)
                            .register_reader();

        SystemName::new(resize_events_id)
    }
}
```

</details>

## Inserting a resource into the `World`

**Note:** If the resource you wish to insert is the result of an expression,
such as a function call, you must surround that expression in quotes, e.g.
`#[system_desc(insert("MyResource::default()"))]`.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{ReadExpect, System, SystemData},
# };
#
pub struct NonDefault;

#[derive(Default, SystemDesc)]
#[system_desc(insert(NonDefault))]
pub struct SystemName;

impl<'a> System<'a> for SystemName {
    type SystemData = ReadExpect<'a, NonDefault>;
    fn run(&mut self, data: Self::SystemData) {}
}
```

<details>
<summary>Generated code</summary>

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     derive::SystemDesc,
#     ecs::{ReadExpect, System, SystemData},
# };
#
# pub struct NonDefault;
#
# #[derive(Default)]
# pub struct SystemName;
#
# impl<'a> System<'a> for SystemName {
#     type SystemData = ReadExpect<'a, NonDefault>;
#     fn run(&mut self, data: Self::SystemData) {}
# }
#
/// Builds a `SystemName`.
#[derive(Debug)]
pub struct SystemNameDesc;

impl Default for SystemNameDesc {
    fn default() -> Self {
        SystemNameDesc {}
    }
}

impl<'a, 'b> ::amethyst::core::SystemDesc<'a, 'b, SystemName> for SystemNameDesc {
    fn build(self, world: &mut ::amethyst::ecs::World) -> SystemName {
        <SystemName as ::amethyst::ecs::System<'_>>::SystemData::setup(world);

        world.insert(NonDefault);

        SystemName::default()
    }
}
```

[Implementing the `SystemDesc` Trait]: ./implementing_the_system_desc_trait.html

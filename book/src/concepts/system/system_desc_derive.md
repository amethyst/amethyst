# `SystemDesc` Derive

The `SystemDesc` derive supports the following cases when generating a `SystemDesc` trait implementation:

- Parameters to pass to the system constructor.
- Fields to skip -- defaulted by the system constructor.
- Registering a `ReaderId` for an `EventChannel<_>` in the `World`.
- Registering a `ReaderId` to a component's `FlaggedStorage`.
- Inserting a resource into the `World`.

If your system initialization use case is not covered, please see the
[Implementing the `SystemDesc` Trait] page.

## Passing parameters to system constructor

```rust
# use amethyst::ecs::System;
pub struct SystemName {
    field_0: u32,
    field_1: String,
}

impl SystemName {
    fn new(field_0: u32, field_1: String) -> Self {
        SystemName { field_0, field_1 }
    }
}
```

<details>
<summary>Generated code</summary>

```rust
# use amethyst::ecs::System;
# 
# pub struct SystemName {
#   field_0: u32,
#   field_1: String,
# }
# 
# impl SystemName {
#   fn new(field_0: u32, field_1: String) -> Self {
#       SystemName { field_0, field_1 }
#   }
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
        SystemName::new(self.field_0, self.field_1)
    }
}
```

</details>

## Fields to skip -- defaulted by the system constructor

```rust
# use amethyst::ecs::System;
# 
pub struct SystemName {
    field_0: u32,
    field_1: String,
}

impl SystemName {
    fn new(field_1: String) -> Self {
        SystemName {
            field_0: 123,
            field_1,
        }
    }
}
# impl System for SystemName {}
```

<details>
<summary>Generated code</summary>

```rust
# use amethyst::ecs::System;
# 
# pub struct SystemName {
#   field_0: u32,
#   field_1: String,
# }
# 
# impl SystemName {
#   fn new(field_1: String) -> Self {
#       SystemName {
#           field_0: 123,
#           field_1,
#       }
#   }
# }
# 
# impl System for SystemName {}
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
        SystemName::new(self.field_1)
    }
}
```

</details>

**Note:** If there are no field parameters, the `SystemDesc` implementation
will call  `SystemName::default()`:

```rust
# use amethyst::ecs::System;
# 
#[derive(Default)]
pub struct SystemName {
    field_0: u32,
}
# impl System for SystemName {}
```

<details>
<summary>Generated code</summary>

```rust
# use amethyst::ecs::System;
# 
# #[derive(Default)]
# pub struct SystemName {
#   field_0: u32,
# }
# 
# impl System for SystemName {}
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
        SystemName::default()
    }
}
```

</details>

## Registering a `ReaderId` for an `EventChannel<_>` in the `World`

```rust
# use amethyst::{
#   ecs::System,
#   shrev::{EventChannel, ReaderId},
#   ui::UiEvent,
# };
# 
pub struct SystemName {
    reader_id: ReaderId<UiEvent>,
}

impl SystemName {
    fn new(reader_id: ReaderId<UiEvent>) -> Self {
        SystemName { reader_id }
    }
}
# impl System for SystemName {}
```

<details>
<summary>Generated code</summary>

```rust
# use amethyst::{
#   ecs::System,
#   shrev::{EventChannel, ReaderId},
#   ui::UiEvent,
# };
# 
# pub struct SystemName {
#   reader_id: ReaderId<UiEvent>,
# }
# 
# impl SystemName {
#   fn new(reader_id: ReaderId<UiEvent>) -> Self {
#       SystemName { reader_id }
#   }
# }
# 
# impl System for SystemName {}
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
        let reader_id = world.fetch_mut::<EventChannel<UiEvent>>().register_reader();

        SystemName::new(reader_id)
    }
}
```

</details>

## Registering a `ReaderId` to a component's `FlaggedStorage`

```rust
# use amethyst::{
#   ecs::System,
#   shrev::{EventChannel, ReaderId},
#   ui::UiResize,
# };
# 
pub struct SystemName {
    resize_events_id: ReaderId<ComponentEvent>,
}

impl SystemName {
    fn new(resize_events_id: ReaderId<ComponentEvent>) -> Self {
        SystemName { resize_events_id }
    }
}
# impl System for SystemName {}
```

<details>
<summary>Generated code</summary>

```rust
# use amethyst::{
#   ecs::System,
#   shrev::{EventChannel, ReaderId},
#   ui::UiResize,
# };
# 
# pub struct SystemName {
#   resize_events_id: ReaderId<ComponentEvent>,
# }
# 
# impl SystemName {
#   fn new(resize_events_id: ReaderId<ComponentEvent>) -> Self {
#       SystemName { resize_events_id }
#   }
# }
# 
# impl System for SystemName {}
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
        let resize_events_id = WriteStorage::<UiResize>::fetch(&world).register_reader();

        SystemName::new(resize_events_id)
    }
}
```

</details>

## Inserting a resource into the `World`

```rust
# use amethyst::ecs::System;
# 
pub struct NonDefault;

#[derive(Default)]
pub struct SystemName;

impl System for SystemName {
    type SystemData = ReadExpect<'a, NonDefault>;
}
```

<details>
<summary>Generated code</summary>

```rust
# use amethyst::ecs::System;
# 
# pub struct NonDefault;
# 
# #[derive(Default)]
# pub struct SystemName;
# 
# impl System for SystemName {
#   type SystemData = ReadExpect<'a, NonDefault>;
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
        world.insert(NonDefault);

        SystemName::default()
    }
}
```

</details>

[implementing the `systemdesc` trait]: ./implementing_the_system_desc_trait.html

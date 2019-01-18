# Test Examples

## Testing a `Bundle`

```rust,edition2018
# extern crate amethyst;
# extern crate amethyst_test;
#
# use amethyst_test::prelude::*;
# use amethyst::{
#     core::bundle::{self, SystemBundle},
#     ecs::prelude::*,
#     prelude::*,
# };
#
# #[derive(Debug)]
# struct ApplicationResource;
#
# #[derive(Debug)]
# struct MySystem;
#
# impl<'s> System<'s> for MySystem {
#     type SystemData = ReadExpect<'s, ApplicationResource>;
#
#     fn run(&mut self, _: Self::SystemData) {}
#
#     fn setup(&mut self, res: &mut Resources) {
#         Self::SystemData::setup(res);
#         res.insert(ApplicationResource);
#     }
# }
#
#[derive(Debug)]
struct MyBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MyBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> bundle::Result<()> {
        // System that adds `ApplicationResource` to the `World`
        builder.add(MySystem, "my_system", &[]);
        Ok(())
    }
}

// #[test]
fn bundle_registers_system_with_resource() {
    assert!(
        AmethystApplication::blank()
            .with_bundle(MyBundle)
            .with_assertion(|world| {
                // The next line would panic if the resource wasn't added.
                world.read_resource::<ApplicationResource>();
            })
            .run()
            .is_ok()
    );
}
#
# fn main() {
#     bundle_registers_system_with_resource();
# }
```

## Testing a `System`

```rust,edition2018
# extern crate amethyst;
# extern crate amethyst_test;
#
# use amethyst_test::prelude::*;
# use amethyst::{
#     ecs::prelude::*,
#     prelude::*,
# };
#
struct MyComponent(pub i32);

impl Component for MyComponent {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug)]
struct MySystem;
impl<'s> System<'s> for MySystem {
    type SystemData = WriteStorage<'s, MyComponent>;
    fn run(&mut self, mut my_component_storage: Self::SystemData) {
        for mut my_component in (&mut my_component_storage).join() {
            my_component.0 += 1
        }
    }
}

// #[test]
fn system_increases_component_value_by_one() {
    assert!(
        AmethystApplication::blank()
            .with_system(MySystem, "my_system", &[])
            .with_effect(|world| {
                let entity = world.create_entity().with(MyComponent(0)).build();
                world.add_resource(EffectReturn(entity));
            })
            .with_assertion(|world| {
                let entity = world.read_resource::<EffectReturn<Entity>>().0.clone();

                let my_component_storage = world.read_storage::<MyComponent>();
                let my_component = my_component_storage
                    .get(entity)
                    .expect("Entity should have a `MyComponent` component.");

                // If the system ran, the value in the `MyComponent` should be 1.
                assert_eq!(1, my_component.0);
            })
            .run()
            .is_ok()
    );
}
#
# fn main() {
#     system_increases_component_value_by_one();
# }
```

### Testing a `System` in a Custom Dispatcher

This is useful when your system must run *after* some setup has been done, for example adding a resource:

```rust,edition2018
# extern crate amethyst;
# extern crate amethyst_test;
#
# use amethyst_test::prelude::*;
# use amethyst::{
#     ecs::prelude::*,
#     prelude::*,
# };
#
// !Default
struct MyResource(pub i32);

#[derive(Debug)]
struct MySystem;

impl<'s> System<'s> for MySystem {
    type SystemData = WriteExpect<'s, MyResource>;

    fn run(&mut self, mut my_resource: Self::SystemData) {
        my_resource.0 += 1
    }
}

// #[test]
fn system_increases_resource_value_by_one() {
    assert!(
        AmethystApplication::blank()
            .with_setup(|world| {
                world.add_resource(MyResource(0));
            })
            .with_system_single(MySystem, "my_system", &[])
            .with_assertion(|world| {
                let my_resource = world.read_resource::<MyResource>();

                // If the system ran, the value in the `MyResource` should be 1.
                assert_eq!(1, my_resource.0);
            })
            .run()
            .is_ok()
    );
}
#
# fn main() {
#     system_increases_resource_value_by_one();
# }
```

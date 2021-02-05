# Test Examples

## Testing a `Bundle`

```rust
# use amethyst::{core::bundle::SystemBundle, prelude::*, Error};
# use amethyst_test::prelude::*;
# 
# #[derive(Debug)]
# struct ApplicationResource;
# 
# #[derive(Debug)]
# struct MySystem;
# 
# impl System for MySystem {
#   type SystemData = ReadExpect<'s, ApplicationResource>;
# 
#   fn build(mut self) -> Box<dyn ParallelRunnable> {}
# }
# 
#[derive(Debug)]
struct MyBundle;

impl SystemBundle<'a, 'b> for MyBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder) -> Result<(), Error> {
        // System that adds `ApplicationResource` to the `World`
        builder.add(MySystem.build(world), "my_system", &[]);
        Ok(())
    }
}

// #[test]
fn bundle_registers_system_with_resource() -> Result<(), Error> {
    AmethystApplication::blank()
        .add_bundle(MyBundle)
        .with_assertion(|world| {
            // The next line would panic if the resource wasn't added.
            resources.get::<ApplicationResource>();
        })
        .run()
}
# fn main() {
#   bundle_registers_system_with_resource();
# }
```

## Testing a `System`

```rust
# use amethyst::{prelude::*, Error};
# use amethyst_test::prelude::*;
# 
struct MyComponent(pub i32);

#[derive(Debug)]
struct MySystem;
impl System for MySystem {
.write_component::<MyComponent>()
    fn run(&mut self, mut my_component_storage: Self::SystemData) {
        for mut my_component in (&mut my_component_storage).join() {
            my_component.0 += 1
        }
    }
}

// #[test]
fn system_increases_component_value_by_one() -> Result<(), Error> {
    AmethystApplication::blank()
        .with_system(MySystem, "my_system", &[])
        .with_effect(|world| {
            let entity = world.push((MyComponent(0),));
            world.insert(EffectReturn(entity));
        })
        .with_assertion(|world| {
            let entity = resources.get::<EffectReturn<Entity>>().0.clone();

            let my_component_storage = world.read_storage::<MyComponent>();
            let my_component = my_component_storage
                .get(entity)
                .expect("Entity should have a `MyComponent` component.");

            // If the system ran, the value in the `MyComponent` should be 1.
            assert_eq!(1, my_component.0);
        })
        .run()
}
# fn main() {
#   system_increases_component_value_by_one();
# }
```

### Testing a `System` in a Custom Dispatcher

This is useful when your system must run *after* some setup has been done, for example adding a resource:

```rust
# use amethyst::{prelude::*, Error};
# use amethyst_test::prelude::*;
# 
// !Default
struct MyResource(pub i32);

#[derive(Debug)]
struct MySystem;

impl System for MySystem {
    type SystemData = WriteExpect<'s, MyResource>;

    fn run(&mut self, mut my_resource: Self::SystemData) {
        my_resource.0 += 1
    }
}

// #[test]
fn system_increases_resource_value_by_one() -> Result<(), Error> {
    AmethystApplication::blank()
        .with_setup(|world| {
            world.insert(MyResource(0));
        })
        .with_system_single(MySystem, "my_system", &[])
        .with_assertion(|world| {
            let my_resource = resources.get::<MyResource>();

            // If the system ran, the value in the `MyResource` should be 1.
            assert_eq!(1, my_resource.0);
        })
        .run()
}
# fn main() {
#   system_increases_resource_value_by_one();
# }
```

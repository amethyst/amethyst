# Resource

## What is a resource?

A resource is any type that stores data that you might need for your game AND that is not specific to an entity.
For example, the score of a pong game is global to the whole game and isn't owned by any of the entities (paddle, ball and even the ui score text).

## Creating a resource

Resources are stored in the `World` container.

Adding a resource to a `World` instance is done like this:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::ecs::World;

struct MyResource {
    pub game_score: i32,
}

fn main() {
    let mut world = World::empty();
    
    let my = MyResource {
        game_score: 0,
    };
    
    world.insert(my);
}
```

## Fetching a resource (from `World`)

Fetching a resource can be done like this:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::World;
# #[derive(Debug, PartialEq)]
# struct MyResource {
#   pub game_score: i32,
# }
# fn main() {
#   let mut world = World::empty();
#   let my = MyResource{
#     game_score: 0,
#   };
#   world.insert(my);
  // try_fetch returns a Option<Fetch<MyResource>>
  let fetched = world.try_fetch::<MyResource>();
  if let Some(fetched_resource) = fetched {
      //dereference Fetch<MyResource> to access data
      assert_eq!(*fetched_resource, MyResource{ game_score: 0, });
  } else {
      println!("No MyResource present in `World`");
  }
# }
```

If you want to get a resource and create it if it doesn't exist:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::World;
# struct MyResource;
# fn main() {
#   let mut world = World::empty();
#   let my = MyResource;
  // If the resource isn't inside `World`, 
  // it will insert the instance we created earlier.
let fetched = world.entry::<MyResource>().or_insert_with(|| my);
# }
```

If you want to change a resource that is already inside of `World`:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::World;
# struct MyResource {
#   pub game_score: i32,
# }
# fn main() {
#   let mut world = World::empty();
#   let my = MyResource{
#     game_score: 0,
#   };
#   world.insert(my);
  // try_fetch_mut returns a Option<FetchMut<MyResource>>
  let fetched = world.try_fetch_mut::<MyResource>();
  if let Some(mut fetched_resource) = fetched {
    assert_eq!(fetched_resource.game_score, 0);
    fetched_resource.game_score = 10;
    assert_eq!(fetched_resource.game_score, 10);
  } else {
    println!("No MyResource present in `World`");
  }
# }
```

Other ways of fetching a resource will be covered in the system section of the book.

## Deleting a resource

There is no method to properly "delete" a resource added to the world.
The usual method to achieve something similar is to add an `Option<MyResource>` and to set it to `None` when you want to delete it.

## Storages, part 2

A `Component`'s `Storage` is a resource.
The components are "attached" to entities, but as said previously, they are not "owned" by the entities at the implementation level.
By storing them into `Storage`s and by having `Storage` be placed inside `World`,
it allows global access to all of the components at runtime with minimal effort.

Actually accessing the components inside `Storage`s will be covered in the world and system sections of the book.

**WARNING:**
If you try to fetch the component directly, you will not get the storage. You will get a `Default::default()` instance of that component.
To get the `Storage` resource that HOLDS all the `MyComponent` instances, you need to fetch `ReadStorage<MyComponent>`.

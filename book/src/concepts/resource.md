# Resource

## What is a resource?

A `Resource` is any type that stores data that you might need for your game AND that is not specific to an entity.
For example, the score of a pong game is global to the whole game and isn't owned by any of the entities (paddle, ball and even the ui score text).

## Creating a resource

Resources are stored in a, well, `Resources` type. This type is usually stored into a `World` instance, which is covered in the next chapter.
Normally you don't create a `Resources` instance yourself. It is usually made by amethyst automatically.

Adding a resource to a `Resources` instance is done like this:
```rust
use amethyst::ecs::{Resources};

struct MyResource {
    pub game_score: i32,
}

fn main() {
    let mut resources = Resources::new();
    
    let my = MyResource {
        game_score: 0,
    };
    
    resources.insert(my);
}
```

## Fetching a resource (from `Resources`)

Fetching a resource can be done like this:
```rust,ignore
// Returns a Option<MyResource>
let fetched = resources.try_fetch::<MyResource>().expect("No MyResource present in Resources");
```

If you want to get a resource and create it if it doesn't exist:
```rust,ignore
// If the resource isn't inside `Resources`, it will insert the instance we created earlier.
let fetched = resources.entry::<MyResource>().or_insert_with(|| my);
```

If you want to change a resource that is already inside of `Resources`:
```rust,ignore
let mut fetched = resources.try_fetch_mut::<MyResource>().expect("No MyResource present in Resources");
```

Other ways of fetching a resource will be covered in the system section of the book.

## Deleting a resource

There is no method to properly "delete" a resource added to the world.
The usual method to achieve something similar is to add an `Option<MyResource>` and to set it to `None` when you want to delete it.

## Storages, part 2

A `Component`'s `Storage` is a resource.
The components are "attached" to entities, but as said previously, they are not "owned" by the entities at the implementation level.
By storing them into `Storage`s and by having `Storage` be placed inside `Resources`,
it allows global access to all of the components at runtime with minimal effort.

Actually accessing the components inside `Storage`s will be covered in the world and system sections of the book.

**WARNING:**
If you try to fetch the component directly, you will not get the storage. You will get a `Default::default()` instance of that component.
To get the `Storage` resource that HOLDS all the `MyComponent` instances, you need to fetch `ReadStorage<MyComponent>`.

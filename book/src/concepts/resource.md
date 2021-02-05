# Resource

## What is a resource?

A resource is any type that stores data that you might need for your game AND that is not specific to an entity.
For example, the score of a pong game is global to the whole game and isn't owned by any of the entities (paddle, ball and even the ui score text).

## Creating a resource

Resources are stored in the `Resources` container.

Adding a resource to a `Resources` instance is done like this:

```rust
use amethyst::ecs::Resources;

struct MyResource {
    pub game_score: i32,
}

fn main() {
    let mut resources = Resources::default();

    let my = MyResource { game_score: 0 };

    resources.insert(my);
}
```

## Fetching a resource (from `Resources`)

Fetching a resource can be done like this:

```rust
# use amethyst::ecs::Resources;
# #[derive(Debug, PartialEq)]
# struct MyResource {
#   pub game_score: i32,
# }
# fn main() {
#   let mut resources = Resources::default();
#   let my = MyResource { game_score: 0 };
    resources.insert(my);
    let fetched = resources.get::<MyResource>(); // returns Option<Fetch<T>>
    if let Some(fetched_resource) = fetched {
        // dereference Fetch<MyResource> to access data
        assert_eq!(*fetched_resource, MyResource { game_score: 0 });
    } else {
        println!("No MyResource present in `Resources`");
    }
# }
```

If you want to get a resource and create it if it doesn't exist:

```rust
# use amethyst::ecs::Resources;
# struct MyResource;
# fn main() {
#   let mut resources = Resources::default();
    // If the resource isn't inside `Resources`,
    // it will insert the instance we created earlier.
    let fetched = resources.get_or_insert_with(|| MyResource);
    // or
    let fetched = resources.get_or_default::<MyResource>();
# }
```

If you want to change a resource that is already inside of `Resources`:

```rust
# use amethyst::ecs::Resources;
# struct MyResource {
#   pub game_score: i32,
# }
# fn main() {
#   let mut resources = Resources::empty();
#   let my = MyResource { game_score: 0 };
#   resources.insert(my);
    // get_mut returns a Option<FetchMut<MyResource>>
    let fetched = resources.get_mut::<MyResource>();
    if let Some(mut fetched_resource) = fetched {
        assert_eq!(fetched_resource.game_score, 0);
        fetched_resource.game_score = 10;
        assert_eq!(fetched_resource.game_score, 10);
    } else {
        println!("No MyResource present in `Resources`");
    }
# }
```

Other ways of fetching a resource will be covered in the system section of the book.

## Deleting a resource

```rust
# use amethyst::ecs::Resources;
# struct MyResource {
#   pub game_score: i32,
# }
# fn main() {
#   let mut resources = Resources::empty();
#   let my = MyResource { game_score: 0 };
#   resources.insert(my);
    resources.remove::<MyResource>();
# }
```

Refer to the [API Documentation][api] for more examples of how to interact with `Resources`

[api]: https://docs.rs/legion/0.3.1/legion/struct.Resources.html

# Dispatcher

## What is a `Dispatcher`?

Dispatchers are the heart of the ECS infrastructure. They are the executors that decide when the `System`s will be executed so that they don't walk over each other.

When a dispatcher is created, it is associated with the systems that it will execute. It then generates an execution plan that respects mutability rules while maximizing parallelism.

## Respecting mutability rules

When a system wants to access a `Storage` or a resource, they can do so either mutably or immutably. This works just like in Rust: either only one system can request something mutably and no other system can access it, or multiple systems can request something but only immutably.

The dispatcher looks at all the `SystemData` in the systems and builds execution stages.

If you want to have the best performance possible, you should prefer immutable over mutable whenever it is possible. (`Read` instead of `Write`, `ReadStorage` instead of `WriteStorage`).

_Note: Please however keep in mind that `Write` is still preferable to interior mutability, such as `Mutex` or `RwLock` for example.


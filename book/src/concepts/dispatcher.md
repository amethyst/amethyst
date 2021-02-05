# Dispatcher

## What is a `Dispatcher`?

Dispatchers are the heart of the ECS infrastructure. They are the executors that decide when the `System`s will be executed so that they don't walk over each other.

When a dispatcher is created, it is associated with the systems that it will execute. It then generates an execution plan that respects mutability rules while maximizing parallelism.

## Respecting mutability rules

When a system wants to access a component or a resource, they can do so either mutably or immutably. This works like in Rust: either only one system can request something mutably and no other system can access it, or multiple systems can request something but only immutably.

The dispatcher builds execution stages based on which components, resources and queries are specified for a system.

If you want to have the best performance possible, you should prefer immutable over mutable whenever it is possible. (`Read` instead of `Write`).

__Note: Please however keep in mind that `Write` is still preferable to locks in most cases, such as `Mutex` or `RwLock` for example.__

# Dispatcher

## What is a `Dispatcher`?

Dispatchers are the heart of the ECS infrastructure. They are the executors that decides when the `System`s will be executed so that they don't walk over each other.

When a dispatcher is created, it is associated with the systems that it will execute. It then generates an execution plan that respects mutability rules while maximizing parallelism.

## Respecting mutability rules

When a system wants to access a `Storage` of a resource, they can do so either mutably or immutably. This works just like in Rust: either only one system can request something mutably and no other system can access it, or multiple systems can request something but only immutably.

The dispatcher looks at all the `SystemData` in the systems and builds execution stages.

If you want to have the best performance possible, you should prefer immutable over mutable whenever it is possible. (`Read` instead of `Write`, `ReadStorage` instead of `WriteStorage`)

## Execution stages

In the context of dispatching, an execution stage is like a pool of systems that are allowed to be ran together so that their `SystemData` respect mutability rules. Execution stages themselves are then ran one after the other.

Dispatching systems this way allows for massive parallelism of game logic. One can create very simple logic blocks as systems and the dispatcher will then take care of running as many as possible at once without needing the user's intervention.

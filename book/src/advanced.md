# Advanced Concepts

In this section, we will go over some of the more advanced concepts use in amethyst.
This section is mostly targeted at people that want a better understanding of the internals of the engine or the usage syntax.

## Lifetimes

There is the concept of lifetimes in Rust.
They are used to indicate how long a reference should live.

Without going into too much details into lifetimes, they are generic types that indicate how long a variable should live depending on the surrounding context at the usage point.
They are similar to how generics works: a generic T can be a float, an integer, or any type that respects the constraints (type bounds) defined on the type.
The difference is that lifetimes use scopes instead of data types.

For example, in Amethyst, the lifetimes for `State<'a,'b>` represent the whole application while `System<'a>` is a single frame, because that's the scope at the place they are used.


Now, in Amethyst, when you create a `State`, you see something like this:
```rust,ignore
impl<'a, 'b> State<'a, 'b> for MyState {}
```

Why are the lifetimes present two times?
First of all, `impl<'a, 'b>` declares that you want your struct implemented for any lifetime that can fit into 'a and 'b (any basically).
Then, `SimpleState<'a,'b>` means you use those lifetimes on the `SimpleState` trait, because the trait requires them.

For more details on how the lifetimes are used internally by such traits, consult the docs and source code available on the website and on github.

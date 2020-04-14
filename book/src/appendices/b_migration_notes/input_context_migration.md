# Input Context Migration Guide

Amethyst recently added a new feature called Input Contexts which breaks backwards compatibility with earlier input files.

If you just want to get on with your day and don't care about this new feature, here's the transformations you need to apply.

```ignore
InputHandler<T> -> InputHandler<(), T>
InputBundle<T> -> InputBundle<(), T>
InputSystem<T> -> InputSystem<(), T>
InputSystemDesc<T> -> InputSystemDesc<(), T>
SdlEventsSystem<T> -> SdlEventsSystem<(), T>
SdlEventsSystemDesc<T> -> SdlEventsSystemDesc<(), T>
FlyControlBundle<T> -> FlyControlBundle<(), T>
FlyMovementSystem<T> -> FlyMovementSystem<(), T>
UiBundle<T> -> UiBundle<(), T>
UiMouseSystem<T> -> UiMouseSystem<(), T>
SelectionMouseSystem<T> -> SelectionMouseSystem<(), T>
SelectionMouseSystemDesc<T> -> SelectionMouseSystemDesc<(), T>
DragWidgetSystem<T> -> DragWidgetSystem<(), T>
```

```ron,ignore
(
    axes: {
        // contents irrelevant
    },
    actions: {
        // contents irrelevant
    }
)
```

Now becomes

```ron,ignore
{
    (): (
        axes: {
            // contents irrelevant
        },
        actions: {
            // contents irrelevant
        }
    )
}
```

If on the other hand you do want to learn about this new feature, please read our new documentation on [Input Contexts](../../input/handling_input.html#input-contexts).
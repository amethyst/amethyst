## Debug Lines

Renders debug lines with a 3D perspective via two separate methods: The
[`DebugLines`] [resource] and the [`DebugLinesComponent`] [component].
Internally, `DebugLines` is just a wrapper around a `DebugLinesComponent`, but
they each have their separate, specific use cases.

The resource method is useful for rendering moving lines such as the purple ones
swinging in the middle of the example scene. This is because the `DebugLines`
resource is cleared immediately after being rendered (see its documentation),
allowing it to be updated easily frame-by-frame.

The component method, however, is useful for rendering persistent lines, such as
the grid pattern shown below. `DebugLinesComponent` is _not_ automatically
cleared on render, so one doesn't have to manually reset it with the same data
every frame if they desire static lines. The [`RenderDebugLines`]
[`RenderPlugin`] includes the necessary rendering [`System`] to automatically
render entities with a `DebugLinesComponent`.

Keybindings:

* `w` - Move forward
* `a` - Move left
* `s` - Move backwards
* `d` - Move left
* `e` - Move upwards
* `q` - Move downwards
* `mouse` - Rotate view

![debug lines example screenshot](./screenshot.png)

[component]: https://docs-src.amethyst.rs/stable/specs/trait.Component.html
[`DebugLines`]: https://docs-src.amethyst.rs/stable/amethyst_rendy/debug_drawing/struct.DebugLines.html
[`DebugLinesComponent`]: https://docs-src.amethyst.rs/stable/amethyst_rendy/debug_drawing/struct.DebugLinesComponent.html
[`RenderDebugLines`]: https://docs-src.amethyst.rs/stable/amethyst_rendy/struct.RenderDebugLines.html
[`RenderPlugin`]: https://docs-src.amethyst.rs/stable/amethyst_rendy/trait.RenderPlugin.html
[resource]: https://book-src.amethyst.rs/master/concepts/resource.html
[`System`]: https://docs-src.amethyst.rs/stable/specs/trait.System.html

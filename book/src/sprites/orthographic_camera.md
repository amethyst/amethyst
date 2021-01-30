# Orthographic Camera

Finally, you need to tell Amethyst to draw in 2D space. This is done by creating an entity with a `Camera` component using orthographic projection. For more information about orthographic projection, refer to the [OpenGL documentation][opengl_ortho].

The following snippet demonstrates how to set up a `Camera` that sees entities within screen bounds, where the entities' Z position is between -10.0 and 10.0:

```rust
use amethyst::{
    core::{math::Orthographic3, transform::Transform},
    prelude::*,
    renderer::camera::Camera,
    window::ScreenDimensions,
};

#[derive(Debug)]
struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, mut data: StateData<'_, GameData>) {
        // ...

        self.initialize_camera(&mut data.world);
    }
}

impl ExampleState {
    fn initialize_camera(&mut self, world: &mut World) {
        let (width, height) = {
            let dim = resources.get::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        // Translate the camera to Z coordinate 10.0, and it looks back toward
        // the origin with depth 20.0
        let mut transform = Transform::default();
        transform.set_translation_xyz(0., height, 10.);

        let camera = Camera::orthographic(0.0, width, 0.0, height, 0.0, 20.0);

        let camera = world.push((transform, camera));
    }
}
```

And you're done! If you would like to see this in practice, check out the [*sprites\_ordered*][ex_ordered] example in the [examples][ex_all] directory.

[ex_all]: https://github.com/amethyst/amethyst/tree/master/examples
[ex_ordered]: https://github.com/amethyst/amethyst/tree/master/examples/sprites_ordered
[opengl_ortho]: https://opengl-notes.readthedocs.io/en/latest/topics/transforms/viewing.html#orthographic-projection

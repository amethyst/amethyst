# Orthographic Camera

Finally, you need to tell Amethyst to draw in 2D space. This is done by creating an entity with a `Camera` component using orthographic projection. For more information about orthographic projection, refer to the [OpenGL documentation][opengl_ortho].

The following snippet demonstrates how to set up a `Camera` that sees entities within screen bounds, without culling entities based on Z position:

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::core::cgmath::{Matrix4, Ortho, Vector3};
use amethyst::core::transform::GlobalTransform;
# use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, Projection, ScreenDimensions
};

#[derive(Debug)]
struct ExampleState;

impl<'a, 'b> SimpleState<'a, 'b> for ExampleState {
    fn on_start(&mut self, mut data: StateData<GameData>) {
        // ...

        self.initialize_camera(&mut data.world);
    }
}

impl ExampleState {
    fn initialize_camera(&mut self, world: &mut World) {
        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        // Camera translation from origin.
        //
        // The Z coordinate of the camera is how far along it should be before it faces
        // the entities. If an entity's Z coordinate is greater than the camera's Z
        // coordinate, it will be culled.
        //
        // By using `::std::f32::MAX` here, we ensure that all entities will be in the
        // camera's view.
        let translation = Matrix4::from_translation(Vector3::new(
            0.0,
            0.0,
            ::std::f32::MAX,
        ));
        let global_transform = GlobalTransform(translation);

        let camera = world
            .create_entity()
            .with(Camera::from(Projection::Orthographic(Ortho {
                left: 0.0,
                right: width,
                top: height,
                bottom: 0.0,
                near: 0.0,
                // The distance that the camera can see. Since the camera is moved to
                // the maximum Z position, we also need to give it maximum Z viewing
                // distance to ensure it can see all entities in front of it.
                far: ::std::f32::MAX,
            }))).with(global_transform)
            .build();
    }
}
```

And you're done! If you would like to see this in practice, check out the [*sprites*][ex_sprites] or [*sprites_ordered*][ex_ordered] examples in the [examples][ex_all] directory.

[ex_all]: https://github.com/amethyst/amethyst/tree/master/examples
[ex_ordered]: https://github.com/amethyst/amethyst/tree/master/examples/sprites_ordered
[ex_sprites]: https://github.com/amethyst/amethyst/tree/master/examples/sprites
[opengl_ortho]: https://opengl-notes.readthedocs.io/en/latest/topics/transforms/viewing.html#orthographic-projection

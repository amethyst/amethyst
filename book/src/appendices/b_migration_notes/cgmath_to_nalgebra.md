# `cgmath` to `nalgebra`

## How To Use

This cheat sheet is split up into the following sections:

- **Type Changes:** Previously you used `this::Type`, now you use `another::Thing`
- **Logic Changes:** Previously you had `object.method(args)`, now you use `object.other(stuff)`

Most changes will have accompanying explanations and code examples on how to switch.

This document is by no means exhaustive, so if there is something missing, or if you can clarify any changes, please correct this!

The text is designed to be searchable, so if you are looking for a specific type or method, please use Ctrl + F: `TypeName`. If you cannot find it in the document, likely we missed it during writing. Please send us a pull request!

## Type Changes

Many types retain the same type name, under the `nalgebra` namespace:

```patch
-use amethyst::core::cgmath::{Vector2, Vector3, Matrix4};
+use amethyst::core::math::{Vector2, Vector3, Matrix4};
```

We will not list the names of every type with the same simple name, but will try to list the changes for types whose simple names are different:

```patch
-cgmath::Ortho
+math::Orthographic3

-cgmath::PerspectiveFov
+math::Perspective3
```

## Logic Changes

- `cgmath` to `nalgebra` functions:

  ```patch
  -Vector3::unit_z()
  +Vector3::z()

  -matrix4.z.truncate()
  +matrix4.column(2).xyz().into()

  -matrix4.transform_point(origin)
  +matrix4.transform_point(&origin)
  ```

- `amethyst::core::transform::Transform`

  - Transformation values are accessed / mutated through accessor methods.

    ```patch
    -transform.translation = Vector3::new(5.0, 2.0, -0.5);
    -transform.scale = Vector3::new(2.0, 2.0, 2.0);
    -transform.rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
    +transform.set_translation_xyz(5.0, 2.0, -0.5);
    +transform.set_scale(2.0, 2.0, 2.0);
    +transform.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0)));

    // Translations
    -transform.translation = Vector3::new(0.0, 0.0, 0.0);
    +*transform.translation_mut() = Vector3::new(0.0, 0.0, 0.0);

    -transform_0.translation - transform_1.translation
    +transform_0.translation() - transform_1.translation()

    -transform.translation[0] = x;
    +transform.set_translation_x(position.x);

    -translation.x += 0.1;
    -translation.y -= 0.1;
    +transform.prepend_translation_x(0.1);
    +transform.prepend_translation_y(-0.1);
    // or
    +transform.translation_mut().x += 0.1;
    +transform.translation_mut().y -= 0.1;

    -let ball_x = transform.translation[0];
    +let ball_x = transform.translation().x;

    -transform.set_position(Vector3::new(6.0, 6.0, -6.0));
    +transform.set_translation_xyz(6.0, 6.0, -6.0);
    // or
    *transform.translation_mut() = Vector3::new(6.0, 6.0, -6.0);

    // Rotations
    -transform.rotation = [1.0, 0.0, 0.0, 0.0].into();
    +use amethyst::core::math::{Quaternion, Unit};
    +
    +*transform.rotation_mut() = Unit::new_normalize(Quaternion::new(
    +    1.0, // w
    +    0.0, // x
    +    0.0, // y
    +    0.0, // z
    +));

    -use amethyst::core::cgmath::Deg;
    -
    -transform.set_rotation(Deg(75.96), Deg(0.0), Deg(0.0));
    +transform.set_rotation_x_axis(1.3257521);
    // or
    +transform.set_rotation_euler(1.3257521, 0.0, 0.0);

    // Scaling
    -transform.scale = Vector3::new(1.0, 1.0, 1.0);
    +*transform.scale_mut() = Vector3::new(1.0, 1.0, 1.0);
    ```

  - `amethyst::core::transform::Transform` prefabs no longer use labels

    ```patch
     // scene.ron
     data: (
         transform: (
    -        translation: (x: 0.0, y: 0.0, z: -4.0),
    -        rotation: (s: 0.0, v: (x: 0.0, y: 1.0, z: 0.0),),
    -        scale: (x: 4.0, y: 2.0, z: 1.0),
    +        translation: (0.0, 0.0, -4.0),
    +        rotation: (0.0, 0.0, 1.0, 0.0),
    +        scale: (4.0, 2.0, 1.0),
         ),
    ```

- `amethyst::renderer::GlobalTransform` inverse.

  ```patch
  -global.0.invert()
  +global.0.try_inverse()
  ```

- `amethyst::renderer::Pos*` fields use `nalgebra` types instead of arrays.

  Type change:

  ```patch
   pub struct PosTex {
  -    pub position: [f32; 3],
  -    pub tex_coord: [f32; 2],
  +    pub position: Vector3<f32>,
  +    pub tex_coord: Vector2<f32>,
   }
  ```

  Usage changes:

  ```patch
   PosTex {
  -    position: [0.0, 0.0, 0.0],
  -    tex_coord: [0.0, 0.0],
  +    position: Vector3::new(0.0, 0.0, 0.0),
  +    tex_coord: Vector2::new(0.0, 0.0),
   }
  ```

- `amethyst::core::math::Matrix4` construction.

  ```patch
  -Matrix4::from_translation(Vector3::new(x, y, z))
  +Matrix4::new_translation(&Vector3::new(x, y, z))

  // OR

  +use amethyst::core::math::Translation3;
  +
  +Translation3::new(x, y, z).to_homogeneous()
  ```

- `UnitQuarternion::rotation_between` is right handed, previously they were left handed.

- Orthographic projection has changed from `(left, right, top, bottom)` to `(left, right, bottom, top)`.

  ```patch
   Projection::orthographic(
       0.0,           // left
       ARENA_WIDTH,   // right
  -    ARENA_HEIGHT,  // top
       0.0,           // bottom
  +    ARENA_HEIGHT,  // top
   )

  -use amethyst::core::cgmath::Ortho;
  -
  -Ortho { left, right, top, bottom, near, far }
  +use amethyst::core::math::Orthographic3;
  +
  +Orthographic3::new(left, right, bottom, top, near, far)
  ```

- Perspective projection

  - Angles are specified in radians:

    ```patch
    use amethyst::renderer::Projection;
    -amethyst::core::cgmath::Deg;

     Projection::perspective(
         1.33333,
    -    Deg(90.0)
    +    std::f32::consts::FRAC_PI_2,
     )
    ```

    ```patch
    // scene.ron
     data: (
         camera: Perspective((
             aspect: 1.3,
    -        fovy: Rad (1.0471975512),
    +        fovy: 1.0471975512,
             // ...
         )),
     )
    ```

  - Prefab fields have been renamed:

    ```patch
    // scene.ron
     data: (
         camera: Perspective((
             aspect: 1.3,
    -        fovy: Rad (1.0471975512),
    +        fovy: 1.0471975512,
    -        near: 0.1,
    -        far: 2000.0,
    +        znear: 0.1,
    +        zfar: 2000.0,
         )),
     )
    ```

- `amethyst::renderer::SpotLight` angle has changed from degrees to radians.

  ```patch
   SpotLight {
  -    angle: 60.0,
  +    angle: std::f32::consts::FRAC_PI_3,
       ..
   }
  ```

- `amethyst::renderer::SunLight` angle has changed from degrees to radians.

  ```patch
   SunLight {
  -    ang_rad: 0.0093,
  +    ang_rad: 0.0093_f32.to_radians(),
       ..
   }
  ```

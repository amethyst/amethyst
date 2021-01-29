# `Transform` API Changes

The names of several `Transform` methods have been changed in order to better reflect their actual function and reduce potential semantic confusion. Many of these changes are breaking and old projects will have to be updated.

## Summary

- `GlobalTransform` has been removed, and merged into `Transform`.
- `set_*` translation methods have been renamed to `set_translation_*`
- `*_local` transforms have been renamed to `append_*`.
- `*_global` transforms have been renamed to `prepend_*`.
- `pitch`, `yaw`, and `roll` methods now use x, y, and z axis respectively.
- `set_position` has been renamed `set_translation`.
- Method aliases for 2D rotation have have been added.
- `set_rotation` methods have been added.
- `euler_angles` method has been added.

## Renamed Transform Methods (Breaking Changes)

- `set(x|y|z)` to `set_translation(x|y|z)`

  ```patch
  -transform.set_x(0.2);
  +transform.set_translation_x(0.2);

  -transform.set_y(7.1);
  +transform.set_translation_y(7.1);

  -transform.set_z(2.4);
  +transform.set_translation_z(2.4);

  -transform.set_xyz(0.2, 1.0, 0.8);
  +transform.set_translation_xyz(0.2, 1.0, 0.8);
  ```

- `_local` to `append_`

  ```patch
  -transform.move_local(Vector3::new(5.0, 2.0, 3.0));
  +transform.append_translation(Vector3::new(5.0, 2.0, 3.0));

  -transform.move_along_local(Vector3::new(3.0, 2.0, 1.0), 4.0);
  +transform.append_translation_along(Vector3::new(3.0, 2.0, 1.0), 8.0);

  -transform.rotate_local(Vector3::new(2.0, 2.1, 3.0), 8.0);
  +transform.append_rotation(Vector3::new(2.0, 2.1, 3.0), 8.0);
  ```

- `(pitch|yaw|roll)_local` to `append_rotation_(x|y|z)_axis`

  ```patch
  -transform.pitch_local(2.3);
  +transform.append_rotation_x_axis(2.3);

  -transform.yaw_local(1.0);
  +transform.append_rotation_y_axis(1.0);

  -transform.roll_local(2.3);
  +transform.append_rotation_z_axis(4.6);
  ```

- `_global` to `prepend_`

  ```patch
  -transform.move_global(Vector3::new(5.0, 2.0, 3.0));
  +transform.prepend_translation(Vector3::new(5.0, 2.0, 3.0));

  -transform.move_along_global(Vector3::new(5.0, 2.0, 3.0));
  +transform.prepend_translation_along(Vector3::new(5.0, 2.0, 3.0));

  -transform.rotate_global(Vector3::new(0.2, 0.4, 0.6), 0.4);
  +transform.prepend_rotation(Vector3::new(0.2, 0.4, 0.6), 0.4);
  ```

- `(pitch|yaw|roll)_global` to `append_rotation_(x|y|z)_axis`

  ```patch
  -transform.pitch_global(0.4);
  +transform.prepend_rotation_x_axis(0.4);

  -transform.yaw_global(0.4);
  +transform.prepend_rotation_y_axis(0.4);

  -transform.roll_global(0.4);
  +transform.prepend_rotation_z_axis(0.4);
  ```

- `translate_(x|y|z)` to `prepend_translation_(x|y|z)`

  ```patch
  -transform.translate_x(3.0);
  +transform.prepend_translation_x(3.0);

  -transform.translate_y(2.4);
  +transform.prepend_translation_y(2.4);

  -transform.translate_z(0.4);
  +transform.prepend_translation_z(0.4);

  -transform.translate_xyz(0.4, 2.4, 3.2);
  +transform.prepend_translation_xyz(0.4, 2.4, 3.2);
  ```

- `set_position` to `set_translation`

  ```patch
  -transform.set_position(Vector3::new(0.3, 0.2, 4.1));
  +transform.set_translation(Vector3::new(0.3, 0.2, 4.1));
  ```

## New Transform Methods

- Set Rotation

  ```rust ,ignore
  transform.set_rotation(UnitQuaternion::identity());
  transform.set_rotation_x_axis(0.4);
  transform.set_rotation_y_axis(2.3);
  transform.set_rotation_z_axis(1.0);
  transform.set_rotation_euler(2.1, 3.4, 8.7);
  ```

- 2D helper functions

  - `rotate_2d`, an alias for `prepend_rotation_z_axis`

    ```rust ,ignore
    transform.rotate_2d(5.0);
    ```

  - `set_rotation_2d`, an alias for `set_rotation_z_axis`

    ```rust ,ignore
    transform.set_rotation_2d(4.7);
    ```

- Euler

  - Get the Euler angles of a transform's rotation.

    ```rust ,ignore
    let (x, y, z) = transform.euler_angles();
    ```

# `Transform` API Changes

The `Transform` API has been changed in order to better reflect their actual function and reduce potential semantic confusion. Many of these changes are breaking and old projects will have to be updated.

## Summary


* `Float` type has been added.
* `GlobalTransform` has been removed.
* `set` methods have been renamed using `set_translation`
* `local` methods have been renamed using `append`
* `pitch`, `yaw`, and `roll` methods now use x, y, and z axis respectively.
* `global` methods have been renamed using `prepend`
* `set_position` has been renamed `set_translation`
* 2D helper aliases have been added
* `set_rotation` methods have been added
* `euler_angles` has been added

## Breaking Changes

### `Float` Type

A wrapper type around f32 and f64. It is used to hide the actual type being used internally. Mostly used with the Transform type. The default type is f32 and you can switch to the f64 type by enabling the "float64" feature gate.

### `set(x|y|z)` to `set_translation(x|y|z)`

```patch
-transform.set_x(0.2);
+transform.set_translation_x(0.2);
```

```patch
-transform.set_y(7.1);
+transform.set_translation_y(7.1);
```

```patch
-transform.set_z(2.4);
+transform.set_translation_z(2.4);
```

```patch
-transform.set_xyz(0.2, 1.0, 0.8);
+transform.set_translation_xyz(0.2);
```

### `_local` to `append_`

```patch
-transform.move_local(Vector3::new(5.0, 2.0, 3.0));
+transform.append_translation(Vector3::new(5.0, 2.0, 3.0));
```

```patch
-transform.move_along_local(Vector3::new(3.0, 2.0, 1.0), 4.0);
+transform.append_translation_along(Vector3::new(3.0, 2.0, 1.0), 8.0);
```

```patch
-transform.rotate_local(Vector3::new(2.0, 2.1, 3.0), 8.0);
+transform.append_rotation(Vector3::new(2.0, 2.1, 3.0), 8.0);
```

### `(pitch|yaw|roll)_local` to `append_rotation_(x|y|z)_axis`

```patch
-transform.pitch_local(2.3);
+transform.append_rotation_x_axis(2.3);
```

```patch
-transform.yaw_local(1.0);
+transform.append_rotation_y_axis(1.0);
```

```patch
-transform.roll_local(2.3);
+transform.append_rotation_z_axis(4.6);
```

### `_global` to `prepend_`

```patch
-transform.move_global(Vector3::new(5.0, 2.0, 3.0));
+transform.prepend_translation(Vector3::new(5.0, 2.0, 3.0));
```

```patch
-transform.move_along_global(Vector3::new(5.0, 2.0, 3.0));
+transform.prepend_translation_along(Vector3::new(5.0, 2.0, 3.0));
```

```patch
-transform.rotate_global(Vector3::new(0.2, 0.4, 0.6), 0.4);
+transform.prepend_rotation(Vector3::new(0.2, 0.4, 0.6), 0.4);
```

### `(pitch|yaw|roll)_global` have changed to `append_rotation_(x|y|z)_axis`

```patch
-pitch_global(0.4);
+prepend_rotation_x_axis(0.4);
```

```patch
-yaw_global(0.4);
+prepend_rotation_y_axis(0.4);
```

```patch
-roll_global(0.4);
+prepend_rotation_z_axis(0.4);
```

### `translate_[xyz]` to `prepend_translation_[xyz]`

```patch
-translate_x(3.0);
+prepend_translation_x(3.0);
```

```patch
-translate_y(2.4);
+prepend_translation_y(2.4);
```


```patch
-translate_z(0.4);
+prepend_translation_z(0.4);
```

```patch
-translate_xyz(0.4, 2.4, 3.2);
+prepend_translation_xyz(0.4, 2.4, 3.2);
```

### `set_position` to `set_translation`

```patch
-transform.set_position(Vector3::new(0.3, 0.2, 4.1));
+transform.set_translation(Vector3::new(0.3, 0.2, 4.1));
```
## New Additions


### Set Rotation

```
transform.set_rotation(UnitQuaternion::identity());
transform.set_rotation_x_axis(0.4);
transform.set_rotation_y_axis(2.3);
transform.set_rotation_z_axis(1.0);
```

### 2D helper functions

`rotate_2d`, an alias for `prepend_rotation_z_axis`
```
transform.rotate_2d(5.0);
```
`set_rotation`, an alias for `set_rotation_z_axis`
```
transform.set_rotation_2d(4.7);
```

### Euler

Get the Euler angles of a transform's rotation.

```
let (x, y, z) = transform.euler_angles();
```

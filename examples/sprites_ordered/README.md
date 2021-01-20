## Sprites Ordered

Draws sprites ordered by Z coordinate. Entities with larger Z coordinates will have their sprites drawn in front of entities with smaller Z coordinates.

This example also demonstrates the use of the `Transparent` component, the depth buffer, and
camera depth values.

Keybindings:

- `T` - Toggle whether the `Transparent` component is attached to entities.
- `R` - Reverse the Z coordinates of the entities.
- `Up` - Increase the Z coordinate of the camera.
- `Down` - Decrease the Z coordinate of the camera.
- `Right` - Increase the depth (Z distance) that the camera can see.
- `Left` - Decrease the depth (Z distance) that the camera can see.

![sprites ordered example screenshot](./screenshot.png)

# Channel

An independent grouping or type of functions that operate on attributes of a component.

Some attributes may be mutated by different functions. These functions can be independent of each other, or they may also be dependent each other. An example of these are translation, scaling, and rotation.

Given the following functions are part of the same animation:

- Translate the object to the right
- Translate the object upwards
- Scale the object up

We want to be able to individually apply related functions, i.e. "apply all translations", "apply all scalings", and "apply all rotations". Each of these groupings is called a **channel**.

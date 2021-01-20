# Animation

Animation in computer graphics can be viewed as controlled mutation of attributes of objects
over time, using a predefined function. Examples of this are:

- Changing coordinates of vertices — movement, scaling up or down
- Changing the hue of a texture — for a "power up" effect

To determine the values each attribute should have at a particular point in time, we define
a set of known values at certain points in the animation — called key frames —
and a function to interpolate the value for the attribute.

This section will guide you in learning how to make use of the animation functionality in Amethyst.

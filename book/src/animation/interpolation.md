# Interpolation

Interpolation is the calculation of an attribute value that lies in between two key frames.

For example, if an object should move in a circle, then we can define an animation that mutates
its X and Y coordinate attributes.

The animation definition can represent this using 5 key frames:

| Key Frame # | X coordinate | Y coordinate |
| ----------- | ------------ | ------------ |
| 0           | 0.0          | 1.0          |
| 1           | 1.0          | 0.0          |
| 2           | 0.0          | -1.0         |
| 3           | -1.0         | 0.0          |
| 4           | 0.0          | 1.0          |

## Non-interpolation

For a perfect circle, the values in between the key frames can be calculated by the `sin(..)`
function for the X coordinate, and the `cos(..)` function for the Y coordinate. So, if we
were trying to calculate what the coordinates should be when `t = 0.5`, we could go `sin( 0.5 * π )`.

However, what if we **do not** have such perfect coordinate control, and we only have
the values at the specified key frames?

## Interpolation

To move in a circle, the X coordinate first increases with a larger step, and the step size
decreases as it approaches the circle boundary on the X axis, where it then flips, and
increases in the negative direction. For the Y coordinate, the magnitude of the step
size increases downwards, then decreases once it has gotten past the halfway point.

The changing step size means, given the first two key frames, 0 and 1, the values do
not change in constant step increments — *linear*ly ([LERP]) —,
but *spherical linear*ly ([SLERP]).

The spherical linear function is a way of saying, given these two key frame values,
and some proportion of time between the two key frames, what should the actual value
be given that the step increments change as they would on a sphere?

## Interpolation Functions

In computer graphics, there are a number of methods commonly used to calculate the interpolated
values. The following functions are available in Amethyst, implemented by the
[`minterpolate`][minterpolate] library, namely:

- Linear
- SphericalLinear
- Step
- CatmullRomSpline
- CubicSpline

Amethyst also allows you to specify your own custom interpolation function.

[lerp]: https://en.wikipedia.org/wiki/Linear_interpolation
[minterpolate]: https://crates.io/crates/minterpolate
[slerp]: https://en.wikipedia.org/wiki/Slerp

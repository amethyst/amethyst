# Sampler

In Amethyst, a `Sampler` is the lowest level working block of an animation. It defines the interpolation function, and what attribute or set of attributes the function mutates.

The `input` holds the timing of the key frames. The `output` holds the values used in the interpolation function for each of the key frames.

You can imagine the interpolation function as `fn(Time) -> ChannelValue`

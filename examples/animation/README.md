## Animation

Animates a sphere using a custom-built animation sampler sequence. Keybindings:

- `Space` - start/pause/unpause the currentanimation(default is translational animation)
- `D` - demonstrate deferred start, translate will run first, then rotate when translate ends, and last scale animation
  will start after rotation has run for 0.66s.
- `T` - set translate to current animation
- `R` - set rotate to current animation
- `S` - set scale to current animation
- `H` - run animation at half speed
- `F` - run animation at full speed
- `V` - run animation at no speed, use stepping keys for controlling the animation
- `Right` - step to the next animation keyframe
- `Left` - step to the previous animation keyframe

![animation example screenshot](./screenshot.png)

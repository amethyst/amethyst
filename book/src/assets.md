# Assets

**Assets** are data that's loaded by a game when it is run. These may be textures, sounds, game level scripts, and so on. In fact, any data that is loaded at runtime may be considered an asset. These are usually stored as files, and distributed alongside the game.

When used well, assets enhance the gaming experience. For example, in an asteroid shooter, when a bullet hits an asteroid we can do the following:

- Draw broken pieces of the asteroid falling away.
- Display a fireball animation.
- Play an explosion sound.

## Handles

In a game, the same asset may be used by different game objects. For example, a fireball texture asset can be used by many different objects that shoot fireballs. Loading the texture mutiple times is an inefficient use of memory; loading it once, and using references to the same loaded asset is *much* more efficient. We call these references, **handles**.

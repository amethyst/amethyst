# Sprites

Sprites are 2D images that represent an object or background. Sprites are represented by two main chunks of data:

* **Texture:** The image made of pixels.
* **Layout:** The coordinates of the sprites on that image.

In Amethyst, these are represented by the [`Texture`][doc_tex] and [`SpriteSheet`][doc_ss] types respectively. The next few pages in this section will explain how to set up your application to load and display sprites.

> **Note:** To see complete application example code, please refer to the [*sprites*][ex_sprites] or [*sprites_ordered*][ex_ordered] examples in the [examples][ex_all] directory.

[doc_ss]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/struct.SpriteSheet.html
[doc_tex]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/struct.Texture.html
[ex_all]: https://github.com/amethyst/amethyst/tree/master/examples
[ex_ordered]: https://github.com/amethyst/amethyst/tree/master/examples/sprites_ordered
[ex_sprites]: https://github.com/amethyst/amethyst/tree/master/examples/sprites

# Sprites

Sprites are 2D images that represent an object or background. Sprites are represented by two main chunks of data:

- **Texture:** The image made of pixels.
- **Sprite Layout:** The (rectangular) coordinates of the sprites on that image.

In Amethyst, these are represented by the [`Texture`][doc_tex] and [`SpriteSheet`][doc_ss] types respectively. The pages in this section will explain how to set up your application to load and display sprites.

> **Note:** The code snippets in this section explain the parts of setting up sprite rendering separately. For complete application examples, please refer to the [*sprites\_ordered*][ex_ordered] example in the [examples][ex_all] directory.

[doc_ss]: https://docs.amethyst.rs/stable/amethyst_rendy/struct.SpriteSheet.html
[doc_tex]: https://docs.amethyst.rs/stable/amethyst_rendy/enum.Texture.html
[ex_all]: https://github.com/amethyst/amethyst/tree/master/examples
[ex_ordered]: https://github.com/amethyst/amethyst/tree/master/examples/sprites_ordered

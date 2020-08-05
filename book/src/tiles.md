# Tiles

Tile maps are a grouping of tiles containing a sprite; they are primarily represented by:

* **TileMap:** A structure for storing tiles and functionality for reading/modifying the map.
* **Tile:** A container for a sprite and other rendering information.

In Amethyst, the [`Tile`][doc_tile] is a trait that must be implemented within you crate which is provided to the [`TileMap`][doc_tilemap] and [`RenderTiles2D`][doc_render]. The pages in this section will explain how to add tile maps to your application. 

> **Note:** The code snippets in this section explain the parts of creating tile maps separately. For complete application examples, please refer to the [*tiles*][ex_tiles] example in the [examples][ex_all] directory.

> **Note:** This section uses [*sprites*](../sprites.md) and assumes knowledge of loading sprites and spritesheets.

[doc_tile]: https://docs.amethyst.rs/stable/amethyst_tiles/trait.Tile.html
[doc_tilemap]: https://docs.amethyst.rs/stable/amethyst_tiles/struct.TileMap.html
[doc_render]: https://docs.amethyst.rs/stable/amethyst_tiles/struct.RenderTiles2D.html
[ex_tiles]: https://github.com/amethyst/amethyst/tree/master/examples/tiles
[ex_all]: https://github.com/amethyst/amethyst/tree/master/examples

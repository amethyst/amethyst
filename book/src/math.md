# Math

Amethyst uses [nalgebra][na] together with [nalgebra-glm][nag] under the hood,
both of which are exposed for us to use. nalgebra is exposed as
`amethyst::core::nalgebra` while nalgebra-glm can be found at the slightly
easier path of `amethyst::math`. As the documentation for both of these projects
are already very good, we won't go into detail here about how to use them. Instead,
we'll redirect you to the excellent [nalgebra website][na] where you can find the
documentation for both of these projects. If you haven't used nalgebra before,
we highly recommend you start out with [nalgebra-glm][nag] as it's somewhat easier
to wrap your head around. If you have ever used [glm] before, you will also feel
very much at home here as nalgebra-glm essentially just implements a very large
subset of of the glm API.

[na]: https://nalgebra.org/
[nag]: https://nalgebra.org/nalgebra_glm/
[glm]: https://glm.g-truc.net/0.9.9/index.html

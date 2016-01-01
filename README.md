# Amethyst

[![Build Status][s1]][tc] [![MIT License][s2]][ml] [![Join the chat][s3]][gc]

[s1]: https://travis-ci.org/ebkalderon/amethyst.svg?branch=master
[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[s3]: https://badges.gitter.im/ebkalderon/amethyst.svg

[ml]: https://github.com/ebkalderon/amethyst/blob/master/COPYING
[tc]: https://travis-ci.org/ebkalderon/amethyst/
[gc]: https://gitter.im/ebkalderon/amethyst?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge

Experimental data-oriented game engine written in Rust. This project is a
*work in progress* and very incomplete; pardon the dust!

# Tutorials

Read the associated [book][bk] for an in-depth guide to using Amethyst. You can
build the book locally with:

[bk]: http://ebkalderon.github.io/amethyst/

```
cargo install mdbook
mdbook build book
```

The text can be found in `book/html/index.html`.

# API Reference

See the online [API reference][ar]. To generate the crate documentation locally,
do:

[ar]: http://ebkalderon.github.io/amethyst/doc/amethyst/index.html

```
cargo doc
```

The API reference can be found in `target/doc/amethyst/index.html`.

# Contributing

Amethyst is an open-source project that values community contribution. Pull
requests are welcome!

We assume you have granted non-exclusive right to your source code under the
[MIT license][ml] and you have processed your code with `rustfmt` prior to
submission. If you want to be known as an author, please add your name to the
AUTHORS.md file in the pull request.

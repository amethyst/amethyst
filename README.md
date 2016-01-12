<img align="left" width="64px" src="./book/images/amethyst_thumb.png" />

# Amethyst

[![Build Status][s1]][tc] [![Crates.io][s2]][ci] [![MIT License][s3]][ml] [![Join the chat][s4]][gc]

[s1]: https://travis-ci.org/ebkalderon/amethyst.svg?branch=master
[s2]: https://img.shields.io/badge/crates.io-0.1.4-orange.svg
[s3]: https://img.shields.io/badge/license-MIT-blue.svg
[s4]: https://badges.gitter.im/ebkalderon/amethyst.svg

[tc]: https://travis-ci.org/ebkalderon/amethyst/
[ci]: https://crates.io/crates/amethyst/
[ml]: https://github.com/ebkalderon/amethyst/blob/master/COPYING
[gc]: https://gitter.im/ebkalderon/amethyst?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge

Experimental data-oriented game engine written in Rust. This project is a *work
in progress* and very incomplete; pardon the dust!

## Usage

Read the [online book][bk] for a comprehensive tutorial to using Amethyst. There
is also an online crate-level [API reference][ar].

[bk]: http://ebkalderon.github.io/amethyst/
[ar]: http://ebkalderon.github.io/amethyst/doc/amethyst/

## Quick Example

See the [Getting Started][gs] chapter in the book for the full-blown "Hello,
World!" tutorial. For the sake of brevity, you can generate an empty game
project with the [amethyst_cli][ac] tool and build it. Follow along below:

[gs]: http://ebkalderon.github.io/amethyst/getting_started.html
[ac]: https://github.com/ebkalderon/amethyst_cli

```
$ cargo install amethyst_cli
$ amethyst new mygame
$ cd mygame
$ amethyst run
```

If everything goes well, you should see "Hello from Amethyst!" print out to the
terminal and abruptly exit.

## Building Documentation

You can build the book locally with:

```
$ cargo install mdbook
$ mdbook build book
```

The text can be found in `book/html/index.html`. To generate the API
documentation locally, do:

```
$ cargo doc
```

The API reference can be found in `target/doc/amethyst/index.html`.

## Contributing

Amethyst is an open-source project that values community contribution. Pull
requests are welcome!

We assume you have granted non-exclusive right to your source code under the
[MIT license][ml] and you have processed your code with `rustfmt` prior to
submission. If you want to be known as an author, please add your name to the
AUTHORS.md file in the pull request.

See the [Development Roadmap][dr] for ideas on what you can hack on.

[dr]: https://github.com/ebkalderon/amethyst/wiki/Roadmap

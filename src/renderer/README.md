# Amethyst - Rendering Engine

[![Build Status][s1]][tc] [![MIT License][s2]][gl] [![Join the chat][s3]][gc]

[s1]: https://api.travis-ci.org/ebkalderon/amethyst.svg
[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[s3]: https://badges.gitter.im/ebkalderon/amethyst.svg

[tc]: https://travis-ci.org/ebkalderon/amethyst/
[gl]: https://github.com/ebkalderon/amethyst/blob/master/COPYING
[gc]: https://gitter.im/ebkalderon/amethyst?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge

High-level rendering engine with multiple backends. This project is a *work in
progress* and is very incomplete. Pardon the dust!

## Proposal

```rust
extern crate amethyst_renderer;

use amethyst_renderer::*;

fn main() {
    let path = RenderPath { ... };
    let mut front = Frontend::new(path);
    let mut back = Backend::new();

    loop {
        let frame = Frame { ... };

        let ir = front.process(frame);
        let back.process(ir);
    }
}
```

# Amethyst - Rendering Engine

[![Build Status][s1]][tc] [![Crates.io][s2]][ci] [![MIT License][s3]][ml] [![Join the chat][s4]][gc]

[s1]: https://api.travis-ci.org/ebkalderon/amethyst.svg
[s2]: https://img.shields.io/badge/crates.io-0.3.1-orange.svg
[s3]: https://img.shields.io/badge/license-MIT-blue.svg
[s4]: https://badges.gitter.im/amethyst/amethyst.svg

[tc]: https://travis-ci.org/ebkalderon/amethyst/
[ci]: https://crates.io/crates/amethyst_renderer/
[ml]: https://github.com/amethyst/amethyst/blob/master/COPYING
[gc]: https://gitter.im/org/amethyst/rooms

High-level rendering engine with multiple backends. This project is a *work in
progress* and is very incomplete. Pardon the dust!

## Proposal

```rust
let mut front = Frontend::new(...);
let mut back = Backend::new(...);

let data = PersistentData { ... };
let handles = back.load_persistent_data(data);

loop {
    let frame = Frame { ... };
    let ir = front.process(frame);
    back.process(ir);
}
```

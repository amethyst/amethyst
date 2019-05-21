# Amethyst Networking

[![Build Status][s2]][l2] [![Latest Version][s1]][l1] [![docs.rs][s4]][l4] [![Join us on Discord][s5]][l5] [![MIT/Apache][s3]][l3]

[s1]: https://img.shields.io/crates/v/amethyst_network.svg
[l1]: https://crates.io/crates/amethyst_network
[s2]: https://jenkins.amethyst-engine.org/buildStatus/icon?job=amethyst%2Fmaster
[l2]: https://jenkins.amethyst-engine.org/job/laminar/job/master/badge/icon
[s3]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[l3]: docs/LICENSE-MIT
[s4]: https://docs.rs/amethyst_network/badge.svg
[l4]: https://docs.rs/amethyst_network/
[s5]: https://img.shields.io/discord/425678876929163284.svg?logo=discord
[l5]: https://discord.gg/GnP5Whs

The networking crate for the `amethyst` game engine. This crate provides the API and functionality which application developers will normally use to develop multiplayer games. The main engine can be found at https://amethyst.rs.

This project is still at an early stage. We are currently designing and implementing a fast/robust networking system on top of specs. To exercise our implementation, we are creating a small test game which we will make public when we feel it's in a good place. Eventually, as we gain more confidence in our solution, we will move stable functionality over from that game to amethyst network.

Currently, amethyst network supports:
- Reliable (ordered, sequenced) UDP.
- Unreliable (sequenced) UDP.
- Connect/Disconnect events from clients.
- Automatic creation of `NetConnection` on client connect.
- Automatic Fragmentation of big packets

We use [laminar](https://github.com/amethyst/laminar) as the application layer communication protocol.

## Contribution

Unless you explicitly state otherwise, any Contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

For more information or help, please come find us on the amethyst discord server's `#net` channel. We are working on architecture, design, and roadmaps and can definitely use some helping hands, don't hessitate :). 

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](docs/LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](docs/LICENSE-MIT) or https://opensource.org/licenses/MIT)
at your option.

# Contributing (WASM edition)

## Links

* [WASM Rush (Project board)](https://github.com/amethyst/amethyst/projects/20)
* [WASM Effort (forum thread)](https://community.amethyst.rs/t/wasm-effort/1336)
* [WASM Issues](https://github.com/amethyst/amethyst/issues?q=is%3Aissue+is%3Aopen+label%3A%22feat%3A+WASM+support%22)
* [Contribution guide](https://github.com/amethyst/amethyst/tree/wasm/docs/CONTRIBUTING_WASM.md)
* [Discord](https://discord.gg/amethyst)

Repositories and branches:

* [amethyst:wasm](https://github.com/amethyst/amethyst/tree/wasm)
* [rendy:jaynus-fixes](https://github.com/amethyst/rendy/tree/jaynus-fixes)
* [winit:clone-events](https://github.com/amethyst/winit/tree/clone-events)
* [gfx:wasm](https://github.com/amethyst/gfx/tree/wasm)
* [glutin:wasm](https://github.com/amethyst/glutin/tree/wasm)

## Development

### Environment Setup

1. Install Rust: https://www.rust-lang.org/tools/install
2. `rustup component add wasm32-unknown-unknown`
3. Install `wasm-pack`: https://rustwasm.github.io/wasm-pack/installer/
4. Install `npm`: https://www.npmjs.com/get-npm

### Ongoing Development

1. Update: `cargo update`
2. Build and test (native): `cargo test --workspace --features "gl"`
3. Build (wasm): `wasm-pack build -- --features "wasm gl"`

## Please Read

1. Work on a feature branch that is branched off the `wasm` branch, and PR back into the `wasm` branch.
2. As much as possible, commits should compile for both native and WASM builds. I made sure the `wasm` branch successfully compiles for both targets before making the call for contributions.
3. Please write meaningful commit messages -- what was done and why it was done -- so someone else is able to understand and pick up where you left off if needed.
4. It's okay to pick up a task, and put it down when it gets hard. Programming is hard. Programming in graphics land for the web is extremely demanding, so chip at it in small doses, and have a break.

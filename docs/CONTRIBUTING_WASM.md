# Contributing (WASM edition)

## Links

* [Contribution guide](https://github.com/amethyst/amethyst/tree/wasm/docs/CONTRIBUTING_WASM.md) -- Guide on how to set up your development environment, and commands to run.
* [WASM Issues](https://github.com/amethyst/amethyst/issues?q=is%3Aissue+is%3Aopen+label%3A%22feat%3A+WASM+support%22) -- Pick something from here to do.
* [WASM Rush (Project Board)](https://github.com/amethyst/amethyst/projects/20) -- Bird's eye view, who's working on what.
* [WASM Effort (Forum Thread)](https://community.amethyst.rs/t/wasm-effort/1336)
* [Discord](https://discord.gg/amethyst) -- Chat with other people on the [`#engine-general` channel](https://discordapp.com/channels/425678876929163284/425679992244928512).

Repositories and branches:

```bash
# End to end POC repository
git clone git@github.com:amethyst/pong_wasm.git

# Crates
git clone git@github.com:amethyst/amethyst.git && (cd amethyst && git checkout wasm)
git clone git@github.com:amethyst/rendy.git && (cd rendy && git checkout wasm)
git clone git@github.com:amethyst/shred.git && (cd shred && git checkout wasm)
git clone git@github.com:amethyst/winit.git && (cd winit && git checkout wasm)
git clone git@github.com:amethyst/gfx.git && (cd gfx && git checkout wasm)
git clone git@github.com:amethyst/glutin.git && (cd glutin && git checkout wasm)
```

* [pong_wasm](https://github.com/amethyst/pong_wasm)
* [amethyst:wasm](https://github.com/amethyst/amethyst/tree/wasm)
* [rendy:wasm](https://github.com/amethyst/rendy/tree/wasm)
* [shred:wasm](https://github.com/amethyst/shred/tree/wasm)
* [winit:wasm](https://github.com/amethyst/winit/tree/wasm)
* [gfx:wasm](https://github.com/amethyst/gfx/tree/wasm)
* [glutin:wasm](https://github.com/amethyst/glutin/tree/wasm)

## Development

### Environment Setup

1. Install Rust: https://www.rust-lang.org/tools/install
2. `rustup target add wasm32-unknown-unknown`
3. Install `wasm-pack`: https://rustwasm.github.io/wasm-pack/installer/
4. Install `npm`: https://www.npmjs.com/get-npm

### Ongoing Development

1. Update: `cargo update`
2. Build and test (native): `cargo test --workspace --features "gl"`
3. Build (wasm): `./scripts/build_wasm.sh`

## Please Read

1. Work on a feature branch that is branched off the `wasm` branch, and PR back into the `wasm` branch.
2. As much as possible, commits should compile for both native and WASM builds. I made sure the `wasm` branch successfully compiles for both targets before making the call for contributions.
3. Please write meaningful commit messages -- what was done and why it was done -- so someone else is able to understand and pick up where you left off if needed.
4. It's okay to pick up a task, and put it down when it gets hard. Programming is hard. Programming in graphics land for the web is extremely demanding, so chip at it in small doses, and have a break.

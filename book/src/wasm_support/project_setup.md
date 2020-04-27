# Project Setup

The following `Cargo.toml` sets up a project to use Amethyst with WASM support.

```toml
# 1. Add or change the `amethyst` dependency to use the Amethyst repository's
#    `"wasm"` branch:
#
#    It is important for `default-features` to be off, as the `"parallel"`
#    feature is enabled by default and is not supported by the WASM target.
[dependencies.amethyst]
git = "https://github.com/amethyst/amethyst.git"
branch = "wasm"
default-features = false
features = [
    "audio",
    "renderer",
    "wav",

    # Optionally include the `"network"` and `"web_socket"` features if the
    # application has online play with the `amethyst_network` crate.
    # "network",
    # "web_socket",

    # Other features required by the project should also be included.
    #
    # See <https://github.com/amethyst/amethyst/blob/wasm/Cargo.toml#L24> for
    # additional features not listed here.
]

# 2. Add WASM specific dependencies for setting the canvas, logging, and panic
#    handling.
[target.'cfg(target_arch = "wasm32")'.dependencies]
console_error_panic_hook = { version = "0.1.6" }
wasm-bindgen = { version = "0.2.60" }
web-sys = { version = "0.3.36" }

# 3. Add features to toggle the graphics backend and WASM support.
[features]
default = ["parallel"]
parallel = ["amethyst/parallel"]
gl = ["amethyst/gl"]
vulkan = ["amethyst/vulkan"]
metal = ["amethyst/metal"]
wasm = ["amethyst/wasm"]
```

# Development Environment

These steps set up a development environment to build a WASM application. This assumes the environment is already set up to develop a Rust application.

These instructions have to be run once per machine.

1. Use the nightly compiler

    ```bash
    rustup default nightly
    ```

2. Add the WASM target:

    ```bash
    rustup target add wasm32-unknown-unknown
    ```

3. Install the WASM bindgen binary:

    ```bash
    cargo install wasm-bindgen-cli
    ```

4. Install a HTTP server application:

    ```bash
    cargo install simple-http-server
    ```

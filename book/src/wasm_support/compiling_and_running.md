# Compiling and Running

These steps build and run the project as a WASM application.

1. Compile the project as a WASM application.

    ```bash
    ./build_release.sh
    ```

    If it succeeds, you should see output similar to the following:

    ```js
    + wasm-bindgen target/wasm32-unknown-unknown/release/myapp.wasm --out-dir pkg --no-modules
    * audio_context_workaround=const lAudioContext = (typeof AudioContext !== 'undefined' ? AudioContext : typeof webkitAudioContext !== 'undefined' ? webkitAudioContext : null)
    * sed -i s/const lAudioContext.\+$/const lAudioContext = (typeof AudioContext !== 'undefined' ? AudioContext : typeof webkitAudioContext !== 'undefined' ? webkitAudioContext : null)/ pkg/myapp.js
    ```

2. Start the HTTP server, disabling caching.

    ```bash
    simple-http-server -i --nocache
    ```

3. Open <http://localhost:8000> using Chrome, and click the launch button to start the application.

    Currently only Chrome is able to run the application, and this has been tested on both Windows and Linux. However, for a more complex game, the application may still crash in Chrome on Windows.

    For Firefox support, please refer to <https://github.com/amethyst/amethyst/issues/2230>.

An example of WASM support can be seen in the [`pong_wasm`] repository.

[`pong_wasm`]: https://github.com/amethyst/pong_wasm

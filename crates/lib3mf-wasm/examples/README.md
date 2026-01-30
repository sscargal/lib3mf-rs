# WASM Example

This example demonstrates how to use the `lib3mf-wasm` bindings in a browser environment.

## Prerequisites

1.  **wasm-pack**: Install it using `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
2.  **Web Server**: You need a local web server (like `python3 -m http.server`, `npx serve`, or `live-server`) to serve the files due to CORS restrictions.

## Running the Example

1.  **Build the WASM module**:
    From the root of the repository, run:
    ```bash
    cd crates/lib3mf-wasm
    wasm-pack build --target web
    ```
    This will create a `pkg/` directory inside `crates/lib3mf-wasm`.

2.  **Prepare the Example Directory**:
    The `index.html` looks for the WASM package in `./pkg`. You can either copy the `pkg` folder into this directory or create a symlink:
    ```bash
    cd examples
    ln -s ../pkg pkg
    ```

3.  **Serve the files**:
    From the `crates/lib3mf-wasm/examples` directory, start a web server:
    ```bash
    python3 -m http.server 8080
    ```

4.  **View in Browser**:
    Open `http://localhost:8080` and upload a `.3mf` file.

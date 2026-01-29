# lib3mf-wasm

WebAssembly bindings for [lib3mf-rs](../lib3mf-core), allowing you to parse and manipulate 3MF files in the browser or Node.js.

## Build

You need `wasm-pack` installed:

```bash
cargo install wasm-pack
```

Build for web:

```bash
wasm-pack build --target web
```

## Usage

### Web

See [examples/index.html](examples/index.html) for a complete example.

```javascript
import init, { WasmModel } from './pkg/lib3mf_wasm.js';

async function run() {
    await init();
    
    // Fetch a 3MF file
    const response = await fetch('model.3mf');
    const buffer = await response.arrayBuffer();
    const bytes = new Uint8Array(buffer);
    
    // Parse
    try {
        const model = WasmModel.from_bytes(bytes);
        console.log("Unit:", model.unit());
        console.log("Objects:", model.object_count());
    } catch (e) {
        console.error("Error parsing 3MF:", e);
    }
}

run();
```

## Features

- Parse 3MF files directly from byte arrays (JS `Uint8Array`).
- Access model metadata (units, resources).
- Fully compatible with `lib3mf-core` capabilities.

## License

MIT

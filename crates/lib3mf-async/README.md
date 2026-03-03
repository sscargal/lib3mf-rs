# lib3mf-async

[![Crates.io](https://img.shields.io/crates/v/lib3mf-async.svg)](https://crates.io/crates/lib3mf-async)
[![docs.rs](https://docs.rs/lib3mf-async/badge.svg)](https://docs.rs/lib3mf-async)
[![License](https://img.shields.io/crates/l/lib3mf-async.svg)](LICENSE)

Non-blocking async 3MF parsing with tokio - high-throughput manufacturing data processing.

## When to Use This Crate

Use `lib3mf-async` when you need:
- Non-blocking I/O for web servers or async applications
- High-throughput processing of multiple 3MF files
- Integration with tokio-based async ecosystems

## Quick Start

```toml
[dependencies]
lib3mf-async = "0.4"
tokio = { version = "1", features = ["full"] }
```

```rust
use lib3mf_async::loader::load_model_async;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = load_model_async("model.3mf").await?;
    println!("Loaded model with {} objects", model.resources.iter_objects().count());
    Ok(())
}
```

## Features

- Async ZIP archive reading using `async-zip`
- Non-blocking I/O operations
- Compatible with Tokio runtime
- Seamless integration with `lib3mf-core` types

## Performance

Async I/O allows processing multiple 3MF files concurrently without blocking:

```rust
use lib3mf_async::loader::load_model_async;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec!["model1.3mf", "model2.3mf", "model3.3mf"];
    let handles: Vec<_> = files.into_iter().map(|path| {
        tokio::spawn(async move {
            load_model_async(path).await
        })
    }).collect();

    let results = futures::future::join_all(handles).await;
    for result in results {
        match result {
            Ok(Ok(model)) => println!("Loaded: {} objects", model.resources.iter_objects().count()),
            _ => eprintln!("Failed to load model"),
        }
    }
    Ok(())
}
```

## Related

- [lib3mf-core](https://crates.io/crates/lib3mf-core) - Core parsing library (required dependency)
- [Full Documentation](https://sscargal.github.io/lib3mf-rs/)

## License

BSD-2-Clause

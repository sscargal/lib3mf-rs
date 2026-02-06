# lib3mf-async

[![Crates.io](https://img.shields.io/crates/v/lib3mf-async.svg)](https://crates.io/crates/lib3mf-async)
[![docs.rs](https://docs.rs/lib3mf-async/badge.svg)](https://docs.rs/lib3mf-async)

Non-blocking async 3MF parsing with tokio - high-throughput manufacturing data processing.

## When to Use This Crate

Use `lib3mf-async` when you need:
- Non-blocking I/O for web servers or async applications
- High-throughput processing of multiple 3MF files
- Integration with tokio-based async ecosystems

## Quick Start

```toml
[dependencies]
lib3mf-async = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use lib3mf_async::AsyncZipArchiver;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let archiver = AsyncZipArchiver::from_file("model.3mf").await?;
    let model = archiver.parse_model().await?;
    println!("Loaded model with {} objects", model.iter_objects().count());
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
use lib3mf_async::AsyncZipArchiver;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Process multiple files concurrently
    let handles: Vec<_> = files.iter().map(|path| {
        tokio::spawn(async move {
            let archiver = AsyncZipArchiver::from_file(path).await?;
            archiver.parse_model().await
        })
    }).collect();

    // Wait for all to complete
    let models = futures::future::join_all(handles).await;
    Ok(())
}
```

## Related

- [lib3mf-core](https://crates.io/crates/lib3mf-core) - Core parsing library (required dependency)
- [Full Documentation](https://sscargal.github.io/lib3mf-rs/)

## License

MIT OR Apache-2.0
